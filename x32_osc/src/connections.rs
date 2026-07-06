use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rosc::{OscPacket, OscMessage, OscType, encoder, decoder, OscError};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::thread::sleep;
use std::time::Duration;

use crate::X32Console;

#[derive(Debug)]
struct OscInterface {
    ip: IpAddr,
    mask: IpAddr,
    broadcast: IpAddr,
}

impl Clone for OscInterface {
    fn clone(&self) -> Self {
        OscInterface {
            ip: self.ip.clone(),
            mask: self.mask.clone(),
            broadcast: self.broadcast.clone(),
        }
    }
}

#[derive(Debug)]
struct OscSocket {
    socket: UdpSocket,
    interface: OscInterface,
}

impl PartialEq for X32Console {
    fn eq(&self, other: &Self) -> bool {
        self.model == other.model && self.ip == other.ip && self.version == other.version
    }
}

pub struct ConnectionManager {
    network_interfaces: Vec<OscInterface>,
    open_sockets: Vec<OscSocket>,
    id_counter: usize,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let itfs = get_search_itfs();

        ConnectionManager {
            network_interfaces: itfs,
            open_sockets: Vec::new(),
            id_counter: 0,
        }
    }
    pub fn scan(&mut self) -> Vec<X32Console> {
        println!("Scanning in connection manager!");
        for itf in &self.network_interfaces {
            //TODO: Starter, this will need to be improved, including checking if something is already open
            match bind_and_config(itf.ip) {
                Ok(sock) => {
                    self.open_sockets.push(OscSocket {
                        socket: sock,
                        interface: itf.clone(),
                    });
                },
                Err(_) => {
                    println!("Couldn't bind to the interface: {:?} or configure it; Skipping.", itf);
                }
            }
        }

        let Ok(scan_packet) = encoder::encode(&OscPacket::Message(OscMessage {
            addr: String::from("/xinfo"),
            args: Vec::new(),
        })) else {
            println!("Failed to create scanning packet; Returning empty list.");
            return Vec::new();
        };

        let mut results: Vec<X32Console> = Vec::new();

        for _ in 0..5 {
            for osc_sock in &self.open_sockets {
                let addr = match osc_sock.interface.broadcast {
                    IpAddr::V4(ipv4) => {
                        SocketAddr::V4(SocketAddrV4::new(ipv4, 10023))
                    },
                    IpAddr::V6(ipv6) => {
                        SocketAddr::V6(SocketAddrV6::new(ipv6, 10023, 0, 0))
                    },
                };
                let sock = &osc_sock.socket;
                let result = sock.send_to(&scan_packet, addr);
                match result {
                    Ok(_) => {
                    },
                    Err(e) => {
                        println!("Error sending to {}: {}", osc_sock.interface.broadcast, e);
                    }
                }
            }

            sleep(Duration::from_secs(1));

            for osc_sock in &self.open_sockets {
                let sock = &osc_sock.socket;
                let mut recv_buf: [u8; 1024] = [0; 1024];
                let Ok(num_bytes) = sock.recv(&mut recv_buf) else {
                    continue;
                };
                let Ok(console) = parse_xinfo(&recv_buf[..num_bytes], &self.id_counter) else {
                    continue;
                };
                if !results.contains(&console) {
                    self.id_counter += 1;
                    results.push(console);
                }
            }
        }

        results
    }
}

fn bind_and_config (ip: IpAddr) -> std::io::Result<UdpSocket> {
    let new_sock = UdpSocket::bind(SocketAddr::new(ip, 0))?;
    new_sock.set_nonblocking(true)?;
    new_sock.set_broadcast(true)?;
    Ok(new_sock)
}

fn parse_xinfo (bytes: &[u8], board_id: &usize) -> Result<X32Console, OscError> {
    let (_, osc_packet) = decoder::decode_udp(bytes)?;
    let OscPacket::Message(mut osc_message) = osc_packet else {
        return Err(OscError::BadMessage("Expected an individual message as an xinfo response"));
    };

    if osc_message.addr != "/xinfo" || osc_message.args.len() != 4 {
        return Err(OscError::BadAddress("Expected /xinfo address with 4 args as a response".to_string()));
    }

    let version = osc_message.args.pop();
    osc_message.args.pop();
    let model = osc_message.args.pop();
    let ip = osc_message.args.pop();

    match (version, model, ip) {
        (Some(OscType::String(version)), Some(OscType::String(model)), Some(OscType::String(ip))) => {
            Ok(X32Console {
                model,
                ip,
                version,
                id: board_id.clone(),
            })
        },
        _ => {
            Err(OscError::BadArg(String::from("Expected args to be strings")))
        }
    }
}

/// Get a list of socket addresses and the corresponding broadcast addresses for the computer
fn get_search_itfs() -> Vec<OscInterface> {
    let network_ifs = NetworkInterface::show().unwrap();

    let mut search_addrs: Vec<OscInterface> = Vec::new();

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
                search_addrs.push(OscInterface {
                    ip: IpAddr::V4(ip),
                    mask: IpAddr::V4(mask),
                    broadcast: IpAddr::V4(Ipv4Addr::from_bits(broadcast)),
                })
            };
        }
    }

    search_addrs
}