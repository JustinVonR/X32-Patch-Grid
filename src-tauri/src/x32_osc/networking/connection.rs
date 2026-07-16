use std::net::SocketAddr;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::AppHandle;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, broadcast};
use tauri::async_runtime::spawn;
use tokio::time::sleep;
use crate::x32_osc::errors::CommandResult;
use super::bind_and_config;

use super::{
    NetInterface,
    super::types::{X32Console},
};

#[derive(Debug)]
pub enum ConnectionMsg {
    Command(),
    Query(),
}

#[derive(Clone, Debug)]
pub enum Response {
    Update(),
    Query(),
}




pub struct ConnectionState {
    event_handle: AppHandle,
    stop: AtomicBool,
    socket: UdpSocket,
}

impl ConnectionState {
    pub fn new(app: AppHandle, socket: UdpSocket) -> Self {
        ConnectionState {
            event_handle: app,
            stop: AtomicBool::new(false),
            socket,
        }
    }
}

pub struct Connection {
    state: Arc<ConnectionState>,
    pub console: X32Console,
    pub interface: NetInterface,
    pub message_tx: mpsc::Sender<ConnectionMsg>,
    pub response_tx: broadcast::Sender<Response>,
}

impl Connection {
    // Create a new connection to an X32 Console and start running its async tasks
    pub async fn new(console: X32Console, interface: NetInterface, app: AppHandle) -> CommandResult<Connection> {
        let _ = tauri::async_runtime::handle();
        println!("Spawning connection!");
        // Bind socket and connect to console IP
        let socket = bind_and_config(interface.ip).await?;
        socket.connect(SocketAddr::new(console.ip, 10023)).await?;
        
        // Set up channels and notifications for async tasks
        let (message_tx, message_rx) = mpsc::channel(128);
        let (response_tx, _) = broadcast::channel(128);

        let new_connection = Self {
            state: Arc::new(ConnectionState::new(app, socket)),
            console,
            interface,
            message_tx,
            response_tx: response_tx.clone(),
        };

        println!("Set up socket successfully!");

        // Spawn three async tasks: Handle incoming on network, Send outgoing on network, and Handle incoming from UI
        let state = new_connection.state.clone();
        spawn(async {
            handle_app_messages(state, message_rx).await;
        });

        let state = new_connection.state.clone();
        spawn(async {
            handle_network_replies(state, response_tx).await;
        });

        let state = new_connection.state.clone();
        spawn(async {
            handle_send_queue(state).await;
        });

        println!("Returning connection struct");

        Ok(new_connection)
    }

    pub fn disconnect(&self) {
        self.state.stop.store(true, Ordering::Relaxed);
    }
}

// Private functions for the three different async handlers
async fn handle_app_messages(state: Arc<ConnectionState>, _message_rx: mpsc::Receiver<ConnectionMsg>) {
    println!("Async 1 Started");
    let mut stop = false;
    while !stop {
        sleep(Duration::from_secs(5)).await;
        println!("Handling app messages");
        stop = state.stop.load(Ordering::Relaxed);
    }
    println!("Stopping Async 1")
}

async fn handle_network_replies(state: Arc<ConnectionState>, _response_tx: broadcast::Sender<Response>) {
    println!("Async 2 Started");
    let mut stop = false;
    while !stop {
        sleep(Duration::from_secs(5)).await;
        println!("Handling network replies");
        stop = state.stop.load(Ordering::Relaxed);
    }
    println!("Stopping Async 2")
}

async fn handle_send_queue(state: Arc<ConnectionState>) {
    println!("Async 3 Started");
    let mut stop = false;
    while !stop {
        sleep(Duration::from_secs(5)).await;
        println!("Handling send queue");
        stop = state.stop.load(Ordering::Relaxed);
    }
    println!("Stopping Async 3")
}