use rosc::{OscMessage, OscPacket, OscType};
use rosc::decoder::decode_udp;
use rosc::encoder::encode;

use serde::Serialize;

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Emitter};

use tokio::net::UdpSocket;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tokio::spawn;

use super::*;
use crate::x32_osc::errors::CommandResult;
use super::bind_and_config;

const SENT_TIMEOUT_SECS: u64 = 20;

type TokioMutex<T> = tokio::sync::Mutex<T>;

use super::{
    NetInterface,
    super::types::{X32Console},
};



enum ConnectionStatus {
    Connected,
    Timeout,
    New,
}

#[derive(Clone, Serialize)]
struct StatusMsg {
    ip: String,
    model: String,
}

#[derive(Debug)]
struct SocketMsg {
    req_type: ReqType,
    first_ack_time: Option<SystemTime>,
    sent_num: u32,
    osc_msg: OscMessage,
}

//-------------- Shared State for Connection Async Work --------------------------------------------------------------//

struct ConnectionState {
    event_handle: AppHandle,
    stop: CancellationToken,
    socket: UdpSocket,
    send_queue: TokioMutex<VecDeque<SocketMsg>>,
    sent: TokioMutex<Vec<SocketMsg>>,
    send_notify: Arc<Notify>,
    last_update: Mutex<SystemTime>,
    status: Mutex<ConnectionStatus>,
}

impl ConnectionState {
    pub fn new(app: AppHandle, socket: UdpSocket) -> Self {
        ConnectionState {
            event_handle: app,
            stop: CancellationToken::new(),
            socket,
            send_queue: TokioMutex::new(VecDeque::new()),
            sent: TokioMutex::new(Vec::new()),
            send_notify: Arc::new(Notify::new()),
            last_update: Mutex::new(SystemTime::now()),
            status: Mutex::new(ConnectionStatus::New),
        }
    }

    pub async fn push_to_send(&self, msg: SocketMsg) {
        let mut queue = self.send_queue.lock().await;
        queue.push_back(msg);
        self.send_notify.notify_one();
    }
}

//--------------- Connection Structure and Methods ---------------------------------------------------------------------//

/// Struct to create and manage a connection to a single X32 console
pub struct Connection {
    shared_state: Arc<ConnectionState>,
    pub console: X32Console,
}

impl Connection {
    // Create a new connection to an X32 Console and start running its async tasks
    pub async fn new<'s>(console: X32Console, interface: NetInterface, app: AppHandle) -> CommandResult<Connection> {
        let _ = tauri::async_runtime::handle();
        // Bind socket and connect to console IP
        let socket = bind_and_config(interface.ip).await?;
        socket.connect(SocketAddr::new(console.ip, 10023)).await?;
        
        let new_connection = Self {
            shared_state: Arc::new(ConnectionState::new(app, socket)),
            console,
        };

        // Spawn three async tasks: Handle incoming on network, Send outgoing on network, and Handle incoming from UI
        let state = new_connection.shared_state.clone();
        spawn(async {status_loop(state).await});

        let state = new_connection.shared_state.clone();
        spawn(async {handle_network_replies(state).await});

        let state = new_connection.shared_state.clone();
        spawn(async {handle_send_queue(state).await});

        Ok(new_connection)
    }

    pub fn disconnect(&self) {
        let _ = self.shared_state.event_handle.emit("disconnect", ());
        self.shared_state.stop.cancel();
    }

    pub async fn send_osc(&self, msg: OscMessage, req_type: ReqType) {
        self.shared_state.push_to_send(SocketMsg {
            first_ack_time: None,
            sent_num: 0,
            req_type,
            osc_msg: msg,
        }).await;
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.disconnect();
    }
}

//---------------- Functions for Async Connection Running --------------------------------------------------------//

// Private functions for the three different async handlers
async fn status_loop(state: Arc<ConnectionState>) {
    let status_msg = encode(&OscPacket::Message(OscMessage{
        addr: String::from("/status"),
        args: Vec::new(),
    })).expect("hardcoded status message should encode");

    // Maybe want to replace this with a subscription instead of xremote
    let xremote_msg = encode(&OscPacket::Message(OscMessage{
        addr: String::from("/xremote"),
        args: Vec::new(),
    })).expect("hardcoded xremote message should encode");

    while !state.stop.is_cancelled() {
        // Don't care about result, we'll resend at the next interval anyway
        let _ = state.socket.send(&status_msg).await;
        let _ = state.socket.send(&xremote_msg).await;

        if let (Ok(last_update), Ok(mut status)) = (state.last_update.lock(), state.status.lock()) {
            if let Ok(elapsed) = last_update.elapsed() {
                if elapsed.as_secs() > 5 {
                    // We resend this on a loop if needed too
                    let _ = state.event_handle.emit("timeout", ());
                    *status = ConnectionStatus::Timeout;
                }
            }
        }

        sleep(Duration::from_secs(4)).await;
    }
}

async fn handle_network_replies(state: Arc<ConnectionState>) {
    while !state.stop.is_cancelled() {
        let mut recv_buf: [u8; 128] = [0; 128];

        // Try to receive again if error occurs, it could be a disconnected board that would re-appear
        let Ok(bytes) = state.socket.recv(&mut recv_buf).await else {continue};

        force_node_osc_compliance(bytes, &mut recv_buf);

        // Skip packet if it fails to decode
        let Ok((_, osc_packet)) = decode_udp(&recv_buf) else {continue};

        match osc_packet {
            OscPacket::Message(msg) => {
                handle_incoming_osc(state.clone(), msg).await;
            },
            // Not currently handling OSC Bundles, I don't think the X32 uses them
            _ => {},
        }

        clean_sent(state.clone()).await;
    }
}

/// Adds a leading / to the buffer if the first 4 bytes are "node". This is
/// necessary to handle the fact that the node response of the X32 is not
/// compliant to the OSC spec.
fn force_node_osc_compliance(bytes: usize, recv_buf: &mut [u8; 128]) -> Vec<u8> {
    let first_four = String::from_utf8_lossy(&recv_buf[0..=4]);
    let mut out = Vec::new();
    if first_four == "node" && bytes + 1 <= recv_buf.len() {
        out.push(b'/');
        out.append(&mut Vec::from(recv_buf))
    }
    out
}

async fn handle_incoming_osc(state: Arc<ConnectionState>, msg: OscMessage) {
    match msg.addr.as_str() {
        "/status" => {
            let (OscType::String(ip), OscType::String(model)) = (msg.args[1].clone(), msg.args[2].clone()) else {
                return;
            };

            let status_msg = StatusMsg {
                ip: String::from(ip),
                model: String::from(model),
            };

            if let Ok(mut status) = state.status.lock() {
                match *status {
                    ConnectionStatus::Connected => {},
                    _ => {
                        *status = ConnectionStatus::Connected;
                        let _ = state.event_handle.emit("connected", status_msg.clone());
                    }
                }
            }

            if let Ok(mut last_update) = state.last_update.lock() {
                *last_update = SystemTime::now();
            }
        },
        "/" => {
            {
                let mut sent = state.sent.lock().await;

                for sent_msg in sent.iter_mut() {
                    if sent_msg.osc_msg == msg && sent_msg.sent_num > 0 {
                        sent_msg.sent_num -= 1;
                        return;
                    }
                }
            }
            {
                let mut send_queue = state.send_queue.lock().await;

                let Some(curr_send) = send_queue.front() else {
                    return;
                };

                if curr_send.osc_msg == msg {
                    let mut acked_message = send_queue.pop_front()
                        .expect("front message existence checked earlier, still holding mutex");

                    acked_message.sent_num -= 1;
                    acked_message.first_ack_time = Some(SystemTime::now());

                    let mut sent = state.sent.lock().await;

                    sent.push(acked_message);
                }
            }
        },
        _ => {
            {
                let mut sent = state.sent.lock().await;

                for sent_msg in sent.iter_mut() {
                    if msg.addr.find(&sent_msg.osc_msg.addr).is_some_and(|idx| {idx == 0}) && sent_msg.sent_num > 0 {
                        sent_msg.sent_num -= 1;
                        return;
                    }
                }
            }
            {
                let mut send_queue = state.send_queue.lock().await;

                let Some(curr_send) = send_queue.front() else {
                    return;
                };

                if msg.addr.find(&curr_send.osc_msg.addr).is_some_and(|idx| {idx == 0}) {
                    let mut acked_message = send_queue.pop_front()
                        .expect("front message existence checked earlier, still holding mutex");

                    let ReqType::Query(ref mut tx_option) = acked_message.req_type else {
                        return;
                    };

                    let Some(resp_tx) = tx_option.take() else {
                        return;
                    };

                    // If other end has disconnected there will never be a way to send the result. Continue as normal.
                    let _ = resp_tx.send(msg.clone());

                    acked_message.sent_num -= 1;
                    acked_message.first_ack_time = Some(SystemTime::now());

                    let mut sent = state.sent.lock().await;

                    sent.push(acked_message);
                }
            }
        },
    };
}

/// Remove old messages and those for which every send has been acknowledged from the sent vector
async fn clean_sent(state: Arc<ConnectionState>) {
    let mut sent = state.sent.lock().await;
    sent.retain(|msg| {
        msg.sent_num > 0 && msg.first_ack_time.is_some_and(|time| {
            time.elapsed().is_ok_and(|duration| {
                duration.as_secs() > SENT_TIMEOUT_SECS
            })
        })
    })
}

async fn handle_send_queue(state: Arc<ConnectionState>) {
    while !state.stop.is_cancelled() {
        state.send_notify.notified().await;

        loop {
            let mut queue = state.send_queue.lock().await;
            let Some(msg) = queue.front_mut() else {break};
            let Ok(message) = encode(&OscPacket::Message(msg.osc_msg.clone())) else {continue};

            //Slow down if many packets have been unacknowledged, prevents overloading issues
            if msg.sent_num > 10 {
                sleep(Duration::from_secs(1)).await;
            }

            // This will be tried again next loop, so don't need to review error
            if let Ok(_) =  state.socket.send(&message).await {
                msg.sent_num += 1;
            };

            if state.stop.is_cancelled() {
                break;
            }

            sleep(Duration::from_millis(100)).await;
        }
    }
}