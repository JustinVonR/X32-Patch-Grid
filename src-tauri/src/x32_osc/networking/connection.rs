use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter};
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, Notify};
use tokio_util::sync::CancellationToken;
use tokio::time::sleep;
use rosc::{OscMessage, OscPacket, OscType};
use rosc::decoder::decode_udp;
use rosc::encoder::encode;
use serde::Serialize;
use tokio::task::JoinSet;
use crate::x32_osc::errors::{CommandError, CommandResult};
use super::bind_and_config;

use super::{
    NetInterface,
    super::types::{X32Console},
};

#[derive(Debug)]
enum ReqType {
    Command,
    Query(oneshot::Sender<OscPacket>),
}

#[derive(Clone, Serialize)]
struct Status {
    ip: String,
    model: String,
}

#[derive(Debug)]
struct SocketMsg {
    req_type: ReqType,
    first_ack_time: Option<SystemTime>,
    sent_num: u32,
    packet: OscPacket,
}

pub struct ConnectionState {
    event_handle: AppHandle,
    stop: CancellationToken,
    socket: UdpSocket,
    send_queue: Mutex<VecDeque<SocketMsg>>,
    send_notify: Arc<Notify>,
    last_update: Mutex<Option<SystemTime>>,
    last_status: Mutex<Option<Status>>
}

impl ConnectionState {
    pub fn new(app: AppHandle, socket: UdpSocket) -> Self {
        ConnectionState {
            event_handle: app,
            stop: CancellationToken::new(),
            socket,
            send_queue: Mutex::new(VecDeque::new()),
            send_notify: Arc::new(Notify::new()),
            last_update: Mutex::new(None),
            last_status: Mutex::new(None),
        }
    }
}

pub struct Connection {
    state: Arc<ConnectionState>,
    async_handles: JoinSet<()>,
    pub console: X32Console,
    pub interface: NetInterface,
}

impl Connection {
    // Create a new connection to an X32 Console and start running its async tasks
    pub async fn new(console: X32Console, interface: NetInterface, app: AppHandle) -> CommandResult<Connection> {
        let _ = tauri::async_runtime::handle();
        // Bind socket and connect to console IP
        let socket = bind_and_config(interface.ip).await?;
        socket.connect(SocketAddr::new(console.ip, 10023)).await?;
        
        let mut new_connection = Self {
            state: Arc::new(ConnectionState::new(app, socket)),
            console,
            interface,
            async_handles: JoinSet::new(),
        };

        // Spawn three async tasks: Handle incoming on network, Send outgoing on network, and Handle incoming from UI
        let state = new_connection.state.clone();
        new_connection.async_handles.spawn(async {status_loop(state).await});

        let state = new_connection.state.clone();
        new_connection.async_handles.spawn(async {handle_network_replies(state).await});

        let state = new_connection.state.clone();
        new_connection.async_handles.spawn(async {handle_send_queue(state).await});

        Ok(new_connection)
    }

    pub fn disconnect(&self) {
        let _ = self.state.event_handle.emit("disconnect", ());
        self.state.stop.cancel();
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.disconnect();
    }
}

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

        // Update status state as needed and send messages to UI
        // TODO: Clean this maybe, it might already be a fairly clean way of doing it?
        if let (Ok(last_status), Ok(mut last_update)) = (state.last_status.lock(), state.last_update.lock()) {
            if let Some(time) = last_update.clone() {
                if let Ok(elapsed) = time.elapsed() {
                    if elapsed.as_secs() > 6 {
                        *last_update = None;

                        // Again, this will be retried on a loop
                        let _ = state.event_handle.emit("timeout", last_status.clone());
                    }
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
        // TODO: Handle this better?
        let Ok(_) = state.socket.recv(&mut recv_buf).await else {continue};

        // Skip packet if it fails to parse
        let Ok(osc_message) = unpack_udp(&recv_buf) else {continue};

        match osc_message.addr.as_str() {
            "/status" => {
                let (OscType::String(ip), OscType::String(model)) = (osc_message.args[1].clone(), osc_message.args[2].clone()) else {
                    continue;
                };

                let status = Status {
                    ip: String::from(ip),
                    model: String::from(model),
                };

                if let (Ok(mut last_status), Ok(mut last_update)) = (state.last_status.lock(), state.last_update.lock()) {
                    if last_update.is_none() {
                        let _ = state.event_handle.emit("connected", status.clone());
                    }

                    *last_status = Some(status);
                    *last_update = Some(SystemTime::now());
                }
            },
            _ => {},
        }
    }
}

fn unpack_udp(msg: &[u8]) -> CommandResult<OscMessage> {
    let (_, osc_packet) = decode_udp(msg)?;
    match osc_packet {
        OscPacket::Message(msg) => {Ok(msg)},
        OscPacket::Bundle(_) => {Err(CommandError::Parse(String::from("Expected a single message")))}
    }
}

async fn handle_send_queue(state: Arc<ConnectionState>) {
    // TODO: This still needs to be implemented
    while !state.stop.is_cancelled() {
        sleep(Duration::from_secs(10)).await;
    }
}