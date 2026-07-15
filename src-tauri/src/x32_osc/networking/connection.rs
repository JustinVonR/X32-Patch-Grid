use std::sync::Arc;

use super::{
    NetInterface,
    super::types::{X32Console},
};

pub struct ConnectionState {
    
}

impl ConnectionState {
    pub fn new() -> Self {
        ConnectionState {}
    }
}

pub struct Connection {
    state: Arc<ConnectionState>,
    pub console: X32Console,
    pub interface: NetInterface,
}

impl Connection {
    pub fn new(console: X32Console, interface: NetInterface) -> Self {
        let new_connection = Self {
            state: Arc::new(ConnectionState::new()),
            console,
            interface,
        };
        
        
        
        new_connection
    }
    
    pub fn disconnect(&self) {
        
    }
}