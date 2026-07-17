use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::{Arc};
use std::time::{Duration, SystemTime};
use tauri::AppHandle;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, oneshot, Mutex, Notify};
use tokio_util::sync::CancellationToken;
use tokio::time::sleep;
use rosc::OscPacket;
use tokio::task::JoinSet;
use crate::x32_osc::errors::CommandResult;
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

#[derive(Debug)]
struct SocketMsg {
    req_type: ReqType,
    first_ack_time: Option<SystemTime>,
    sent_num: u32,
    packet: OscPacket,
}

#[derive(Debug)]
pub enum ConnectionMsg {
    Command(),
    Query(),
}

pub struct ConnectionState {
    event_handle: AppHandle,
    stop: CancellationToken,
    socket: UdpSocket,
    send_queue: Mutex<VecDeque<SocketMsg>>,
    send_notify: Arc<Notify>,
}

impl ConnectionState {
    pub fn new(app: AppHandle, socket: UdpSocket) -> Self {
        ConnectionState {
            event_handle: app,
            stop: CancellationToken::new(),
            socket,
            send_queue: Mutex::new(VecDeque::new()),
            send_notify: Arc::new(Notify::new()),
        }
    }
}

pub struct Connection {
    state: Arc<ConnectionState>,
    message_tx: mpsc::Sender<ConnectionMsg>,
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
        
        // Set up channels and notifications for async tasks
        let (message_tx, message_rx) = mpsc::channel(128);

        let mut new_connection = Self {
            state: Arc::new(ConnectionState::new(app, socket)),
            console,
            interface,
            message_tx,
            async_handles: JoinSet::new(),
        };

        // Spawn three async tasks: Handle incoming on network, Send outgoing on network, and Handle incoming from UI
        let state = new_connection.state.clone();
        new_connection.async_handles.spawn(async {handle_app_messages(state, message_rx).await});

        let state = new_connection.state.clone();
        new_connection.async_handles.spawn(async {handle_network_replies(state).await});

        let state = new_connection.state.clone();
        new_connection.async_handles.spawn(async {handle_send_queue(state).await});

        Ok(new_connection)
    }

    pub fn disconnect(&self) {
        self.state.stop.cancel();
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.disconnect();
    }
}

// Private functions for the three different async handlers
async fn handle_app_messages(state: Arc<ConnectionState>, _message_rx: mpsc::Receiver<ConnectionMsg>) {
    println!("Async 1 Started");
    while !state.stop.is_cancelled() {
        sleep(Duration::from_secs(5)).await;
        println!("Handling app messages");
    }
    println!("Stopping Async 1")
}

async fn handle_network_replies(state: Arc<ConnectionState>) {
    println!("Async 2 Started");
    while !state.stop.is_cancelled() {
        sleep(Duration::from_secs(5)).await;
        println!("Handling network replies");
    }
    println!("Stopping Async 2")
}

async fn handle_send_queue(state: Arc<ConnectionState>) {
    println!("Async 3 Started");
    while !state.stop.is_cancelled() {
        sleep(Duration::from_secs(5)).await;
        println!("Handling send queue");
    }
    println!("Stopping Async 3")
}