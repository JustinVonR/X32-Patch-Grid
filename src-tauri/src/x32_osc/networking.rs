mod connection;

use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rosc::{OscPacket, OscMessage, OscType, encoder, decoder, OscError};

use std::collections::HashMap;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::sleep;

use rand::RngExt;
use tauri::AppHandle;
use connection::Connection;
use super::{X32Console, ConnectionList};
use super::errors::{CommandError, CommandResult};

//------------------------------ Private Connection Management Types ---------------------------//
#[derive(Debug)]
struct NetInterface {
    ip: IpAddr,
    mask: IpAddr,
    broadcast: IpAddr,
}

impl NetInterface {
    /// Check whether subnet of a remote address matches the subnet of this interface. Always returns false for Ipv6
    fn reaches_ip(&self, remote: IpAddr) -> bool {
        // TODO: Make this handle IPv6?
        let (IpAddr::V4(ip), IpAddr::V4(mask), IpAddr::V4(remote)) = (self.ip, self.mask, remote) else {
            return false;
        };
        let ip_subnet = Ipv4Addr::from_bits(ip.to_bits() & mask.to_bits());
        let remote_subnet = Ipv4Addr::from_bits(remote.to_bits() & mask.to_bits());
        ip_subnet == remote_subnet
    }
}

impl Clone for NetInterface {
    fn clone(&self) -> Self {
        NetInterface {
            ip: self.ip.clone(),
            mask: self.mask.clone(),
            broadcast: self.broadcast.clone(),
        }
    }
}

#[derive(Debug)]
struct OscSocket {
    socket: UdpSocket,
    interface: NetInterface,
}

//------------------------------ Define Connection Manager Object ------------------------------//

pub struct ConnectionManager {
    network_interfaces: Vec<NetInterface>,
    curr_connection: Arc<Mutex<Option<Connection>>>,
    discovered: Arc<Mutex<HashMap<u32, X32Console>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let itfs = get_search_itfs();

        ConnectionManager {
            network_interfaces: itfs,
            curr_connection: Arc::new(Mutex::new(None)),
            discovered: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_connection_list(&self) -> ConnectionList {
        if let (Ok(discovered), Ok(connection)) = (self.discovered.lock(), self.curr_connection.lock()) {
            ConnectionList {
                consoles: discovered.values().cloned().collect(),
                connected_id: match &*connection {
                    Some(con) => Some(con.console.id),
                    None => None,
                }
            }
        } else {
            ConnectionList {
                consoles: Vec::new(),
                connected_id: None,
            }
        }
    }

    pub async fn scan(&self) {
        // Open Sockets for Scanning
        let mut open_sockets: Vec<OscSocket> = Vec::new();

        for itf in &self.network_interfaces {
            match bind_and_config(itf.ip).await {
                Ok(sock) => {
                    open_sockets.push(OscSocket {
                        socket: sock,
                        interface: itf.clone(),
                    });
                },
                Err(_) => {
                    println!("Couldn't bind to the interface: {:?} or configure it; Skipping.", itf);
                }
            }
        }

        // Create xinfo message to get console IP responses
        let Ok(scan_packet) = encoder::encode(&OscPacket::Message(OscMessage {
            addr: String::from("/xinfo"),
            args: Vec::new(),
        })) else {
            println!("Failed to create scanning packet; Returning empty list.");
            return;
        };

        let mut last_scan_ids: Vec<u32> = Vec::new();

        // Send out scanning packet and make list of unique responses
        for _ in 0..5 {
            for osc_sock in &open_sockets {
                let addr = match osc_sock.interface.broadcast {
                    IpAddr::V4(ipv4) => {
                        SocketAddr::V4(SocketAddrV4::new(ipv4, 10023))
                    },
                    IpAddr::V6(ipv6) => {
                        SocketAddr::V6(SocketAddrV6::new(ipv6, 10023, 0, 0))
                    },
                };
                let sock = &osc_sock.socket;
                let result = sock.send_to(&scan_packet, addr).await;
                match result {
                    Ok(_) => {
                    },
                    Err(e) => {
                        println!("Error sending to {}: {}", osc_sock.interface.broadcast, e);
                    }
                }
            }

            sleep(Duration::from_secs(1)).await;

            for osc_sock in &open_sockets {
                let sock = &osc_sock.socket;
                let mut recv_buf: [u8; 1024] = [0; 1024];
                let Ok(num_bytes) = sock.try_recv(&mut recv_buf) else {
                    continue;
                };
                // Only proceed to add board if we can make an ID for it
                if let Ok(id) = &self.gen_id() {
                    // Skip if unable to parse packet data
                    let Ok(console) = parse_xinfo(&recv_buf[..num_bytes], id) else {
                        continue;
                    };
                    // Skip if unable to lock Mutex on discovery list
                    if let Ok(mut discovered) = self.discovered.lock() {
                        let discovered_consoles: Vec<&X32Console> = discovered.values().collect();
                        // Mark ID as seen this scan cycle
                        last_scan_ids.push(console.id);
                        // Skip adding if already present
                        if !discovered_consoles.contains(&&console) {
                            discovered.insert(console.id, console);
                        }

                        drop(discovered);
                    }
                }
            }
        }

        // Remove options not seen this scan unless currently connected
        if let (Ok(mut discovered), Ok(connection)) = (self.discovered.lock(), self.curr_connection.lock()) {
            discovered.retain(|&k, _| {
                last_scan_ids.contains(&k) || match &*connection {
                    Some(con) if con.console.id == k => true,
                    _ => false,
                }
            })
        }

    }

    // Creates a new connection to the discovered console with the specified ID
    pub async fn connect(&self, id: u32, app: AppHandle) -> CommandResult<()> {
        let (discovered, mut connection) = (self.discovered.lock()?, self.curr_connection.lock()?);

        // Only connect if id is still valid
        if discovered.contains_key(&id) {
            let console = discovered.get(&id).expect("already checked ID in discovered");

            let Some(interface) = self.network_interfaces.iter().find(|&i| {i.reaches_ip(console.ip)}) else {
                return Err(CommandError::InvalidOp(String::from("unable to reach console")));
            };

            if let Some(con) = &*connection {
                con.disconnect();
            }

            // Create new connection, starts async automatically
            let new_con = Connection::new(console.clone(), interface.clone(), app)?;
            *connection = Some(new_con);
        } else {
            return Err(CommandError::InvalidOp(String::from("console not available")));
        }
        Ok(())
    }

    // Disconnects the current console connection
    pub fn disconnect(&self) {
        let Ok(mut connection) = self.curr_connection.lock() else {
            return;
        };

        if let Some(connection) = &*connection {
            connection.disconnect();
        }
    }

    fn gen_id(&self) -> CommandResult<u32> {
        let Ok(discovered) = self.discovered.lock() else {
            return Err(CommandError::Mutex);
        };
        // Probably this isn't the best way of generating unique IDs, but it should
        // be okay here
        let mut rng = rand::rng();

        let mut rand = rng.random::<u32>();
        let mut tries = 0;
        while discovered.contains_key(&rand) && tries < 20 {
            rand = rng.random::<u32>();
            tries += 1;
        }

        Ok(rand)
    }
}


//------------------------------ Private Helper Functions --------------------------------------//

async fn bind_and_config (ip: IpAddr) -> std::io::Result<UdpSocket> {
    let new_sock = UdpSocket::bind(SocketAddr::new(ip, 0)).await?;
    new_sock.set_broadcast(true)?;
    Ok(new_sock)
}

fn parse_xinfo (bytes: &[u8], board_id: &u32) -> CommandResult<X32Console> {
    let (_, osc_packet) = decoder::decode_udp(bytes)?;
    let OscPacket::Message(mut osc_message) = osc_packet else {
        return Err(CommandError::OSC(OscError::BadMessage("Expected an individual message as an xinfo response")));
    };

    if osc_message.addr != "/xinfo" || osc_message.args.len() != 4 {
        return Err(CommandError::OSC(OscError::BadAddress(String::from("Expected /xinfo address with 4 args as a response"))));
    }

    let version = osc_message.args.pop();
    osc_message.args.pop();
    let model = osc_message.args.pop();
    let ip = osc_message.args.pop();

    let (Some(OscType::String(version)), Some(OscType::String(model)), Some(OscType::String(ip))) = (version, model, ip) else {
        return Err(CommandError::OSC(OscError::BadArg(String::from("Expected args to be strings"))));
    };

    let ip = IpAddr::V4(Ipv4Addr::from_str(&ip)?);

    Ok(X32Console {
        model,
        ip,
        version,
        id: board_id.clone(),
    })
}

/// Get a list of socket addresses and the corresponding broadcast addresses for the computer
fn get_search_itfs() -> Vec<NetInterface> {
    let network_ifs = NetworkInterface::show().unwrap();

    let mut search_addrs: Vec<NetInterface> = Vec::new();

    // Add broadcast addresses for find board OSC servers. Check on all addresses of all interfaces
    for itf in network_ifs.iter() {
        for i in 0.. itf.addr.len() {
            let ip = itf.addr[i].ip();
            // TODO: Make code handle IPv6 Cases??
            let IpAddr::V4(ip) = ip else {
                continue;
            };

            // Reject if local link, mainly to avoid picking up on things like Bluetooth connections. A valid board would be a separate
            // through a router's DHCP or on a loopback interface
            if ip.is_link_local() {
                continue;
            }

            if let Some(IpAddr::V4(mask)) = itf.addr[i].netmask() {
                let inv_mask = !mask.to_bits();
                let broadcast = ip.to_bits() | inv_mask;
                search_addrs.push(NetInterface {
                    ip: IpAddr::V4(ip),
                    mask: IpAddr::V4(mask),
                    broadcast: IpAddr::V4(Ipv4Addr::from_bits(broadcast)),
                })
            };
        }
    }

    search_addrs
}