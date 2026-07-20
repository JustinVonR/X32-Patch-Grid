use std::net::IpAddr;
use rosc::{OscMessage, OscType};
use serde::Serialize;
use tokio::sync::oneshot;
use crate::x32_osc::errors::{CommandError, CommandResult};

#[derive(Debug, Serialize)]
pub struct X32Console {
    pub model: String,
    pub ip: IpAddr,
    pub version: String,
    pub id: u32,
}

impl Clone for X32Console {
    fn clone(&self) -> Self {
        X32Console {
            model: self.model.clone(),
            ip: self.ip.clone(),
            version: self.version.clone(),
            id: self.id,
        }
    }
}

impl PartialEq for X32Console {
    fn eq(&self, other: &Self) -> bool {
        self.model == other.model && self.ip == other.ip && self.version == other.version
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionList {
    pub consoles: Vec<X32Console>,
    pub connected_id: Option<u32>,
}

#[derive(Debug)]
pub enum ReqType {
    Command,
    Query(Option<oneshot::Sender<OscMessage>>),
}

const ALLOWED_PARAM_ADDRS: [&str; 4] = [
    "/xinfo",
    "/xremote",
    "/config/routing/",
    "/config/userrout/",
];


/// Return whether the address starts with an allowed path for this application
fn match_addr(addr: &String) -> bool {
    for allowed in ALLOWED_PARAM_ADDRS {
        if let Some(0) = addr.find(&allowed) {
            return true;
        }
    }
    false
}

/// Re-packages an OscMessage to ensure that the connection can use it properly.
pub struct X32OscMessage {
    message: OscMessage,
}

//TODO: This is not an exhaustive validation of exact commands, only a rough check
impl X32OscMessage {
    /// Puts the input OscMessage into an X32OscMessage after packaging any message with args (assumed to be a set command rather than a request) as
    /// a node set command with path "/". This ensures it will be acknowledged by the other side of the connection
    pub fn new(msg: OscMessage) -> CommandResult<X32OscMessage> {
        if msg.addr == "/" {
            if msg.args.len() == 1 && matches![msg.args[1], OscType::String(_)] {
                Ok(X32OscMessage {
                    message: msg,
                })
            } else {
                Err(CommandError::InvalidOp(String::from("Set node command should have exactly one string as an argument")))
            }
        } else if match_addr(&msg.addr) {
            // Repackage as a node set if trying to set parameters, that way the board echoes it
            if msg.args.len() >= 1 {
                let mut node_string = String::from("");

                node_string.push_str(&msg.addr);

                for arg in msg.args {
                    match arg {
                        OscType::Int(i) => node_string.push_str(&(" ".to_string() + &i.to_string())),
                        _ => return Err(CommandError::InvalidOp(String::from("Unsupported OSC type for X32 IO Operations"))),
                    }
                }

                Ok(X32OscMessage {
                    message: OscMessage {
                        addr: String::from("/"),
                        args: vec![OscType::String(node_string)],
                    }
                })
            } else {
                Ok(X32OscMessage {
                    message: msg,
                })
            }
        } else {
            Err(CommandError::InvalidOp(String::from("Invalid OSC address for X32 IO Operations")))
        }
    }

    pub fn get_message(&self) -> OscMessage {
        self.message.clone()
    }
}

