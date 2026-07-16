use std::error::Error;
use std::sync::{Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;
use tauri::async_runtime::block_on;
use tokio::net::UdpSocket;
use crate::x32_osc::errors::{CommandError, CommandResult};
use super::bind_and_config;

use super::{
    NetInterface,
    super::types::{X32Console},
};

pub struct ConnectionState {
    event_handle: AppHandle,
    stop: AtomicBool,
}

impl ConnectionState {
    pub fn new(app: AppHandle) -> Self {
        ConnectionState {
            event_handle: app,
            stop: AtomicBool::new(false),
        }
    }
}

pub struct Connection {
    state: Arc<ConnectionState>,
    pub console: X32Console,
    pub interface: NetInterface,
}

impl Connection {
    // Create a new connection to an X32 Console and start running its async tasks
    pub fn new(console: X32Console, interface: NetInterface, app: AppHandle) -> CommandResult<Connection> {
        // Create connection structure
        let new_connection = Self {
            state: Arc::new(ConnectionState::new(app)),
            console,
            interface,
        };
        
        // Bind socket and connect to console IP
        let socket = block_on(bind_and_config(new_connection.interface.ip))?;
        
        // Set up channels and notifications for async tasks
        
        // Spawn three async tasks: Handle incoming on network, Send outgoing on network, and Handle incoming from UI
        
        Ok(new_connection)
    }

    pub fn disconnect(&self) {
        self.state.stop.store(true, Ordering::Relaxed);
    }
}