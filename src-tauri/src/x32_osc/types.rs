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

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use super::*;

    mod x32_console {
        use std::net::Ipv4Addr;
        use std::net::IpAddr;

        use super::*;

        #[test]
        fn console_equality() {
            let console_a = X32Console {
                model: String::from("Test X32"),
                ip: IpAddr::V4(Ipv4Addr::from_str("192.168.1.2").expect("String should be valid IP")),
                version: String::from("4.06"),
                id: 0,
            };

            let console_b = X32Console {
                model: String::from("Test X32"),
                ip: IpAddr::V4(Ipv4Addr::from_str("192.168.1.2").expect("String should be valid IP")),
                version: String::from("4.06"),
                id: 1,
            };

            assert_eq!(console_a, console_b);
        }

        #[test]
        fn console_inequality() {
            let console_a = X32Console {
                model: String::from("Test X32"),
                ip: IpAddr::V4(Ipv4Addr::from_str("192.168.1.2").expect("String should be valid IP")),
                version: String::from("4.06"),
                id: 0,
            };

            let console_b = X32Console {
                model: String::from("X32 Emulator"),
                ip: IpAddr::V4(Ipv4Addr::from_str("192.168.1.2").expect("String should be valid IP")),
                version: String::from("4.06"),
                id: 1,
            };

            let console_c = X32Console {
                model: String::from("Test X32"),
                ip: IpAddr::V4(Ipv4Addr::from_str("192.168.1.6").expect("String should be valid IP")),
                version: String::from("4.06"),
                id: 3,
            };

            let console_d = X32Console {
                model: String::from("Test X32"),
                ip: IpAddr::V4(Ipv4Addr::from_str("192.168.1.2").expect("String should be valid IP")),
                version: String::from("3.19"),
                id: 4,
            };

            assert_ne!(console_a, console_b);
            assert_ne!(console_a, console_c);
            assert_ne!(console_a, console_d);
        }
    }

    mod x32_osc_command {
        use super::*;

        const fn remove_slash(s: & 'static str) -> & 'static str {
            let s = s.as_bytes().split_at(1).1;
            match core::str::from_utf8(s) {
                Ok(s) => s,
                Err(_) => panic!(),
            }
        }

        const VALID_CMDS: &[(&str, &[OscType])] = [
            ("/config/userrout/out/01", [].as_slice()),
            ("/config/userrout/out/02", [].as_slice()),
            ("/config/userrout/in/01", [].as_slice()),
            ("/config/userrout/in/05", [].as_slice()),
            ("/config/routing/IN/1-8", [].as_slice()),
            ("/config/routing/IN/9-16", [].as_slice()),
            ("/config/routing/IN/AUX", [].as_slice()),
            ("/config/routing/AES50A/1-8", [].as_slice()),
            ("/config/routing/AES50A/9-16", [].as_slice()),
            ("/config/routing/AES50B/1-8", [].as_slice()),
            ("/config/routing/AES50B/9-16", [].as_slice()),
            ("/config/routing/CARD/1-8", [].as_slice()),
            ("/config/routing/CARD/9-16", [].as_slice()),
            ("/config/routing/OUT/1-4", [].as_slice()),
            ("/config/routing/OUT/5-8", [].as_slice()),
            ("/config/routing/IN", [].as_slice()),
        ].as_slice();


        const INVALID_CMDS: &[(&str, &[OscType])] = [
            ("/config/userrout/out/00", [].as_slice()),
            ("/config/userrout/out/50", [].as_slice()),
            ("/config/userrout/in/00", [].as_slice()),
            ("/config/userrout/in/35", [].as_slice()),
            ("/config/routing/IN/1-9", [].as_slice()),
            ("/config/routing/IN/8-17", [].as_slice()),
            ("/config/routing/IN/AUXS", [].as_slice()),
            ("/config/routing/AES50C/1-8", [].as_slice()),
            ("/config/routing/AES50C6", [].as_slice()),
            ("/config/routing/AES50B/1-9", [].as_slice()),
            ("/config/routing/AES50B/8-16", [].as_slice()),
            ("/config/routing/CARD/1-10", [].as_slice()),
            ("/config/routing/CARD/9-18", [].as_slice()),
            ("/config/routing/OUT/1-8", [].as_slice()),
            ("/config/routing/OUT/9-16", [].as_slice()),
            ("/config/routing/OUT/", [].as_slice()),
            ("/config/routing/OUT/1-4/", [].as_slice()),
            ("/ch/01/gate/on", [].as_slice()),
        ].as_slice();

        const VALID_NO_SLASH_CMDS: [(&str, &[OscType]); VALID_CMDS.len()] = {
            let mut out: [(&str, &[OscType]); VALID_CMDS.len()] = [("", &[OscType::Nil]); VALID_CMDS.len()];
            let mut i = 0;
            while i < VALID_CMDS.len() {
                let cmd = &VALID_CMDS[i];
                out[i] = (remove_slash(cmd.0), cmd.1);
                i += 1;
            }
            out
        };

        const INVALID_NO_SLASH_CMDS: [(&str, &[OscType]); INVALID_CMDS.len()] = {
            let mut out: [(&str, &[OscType]); INVALID_CMDS.len()] = [("", &[OscType::Nil]); INVALID_CMDS.len()];
            let mut i = 0;
            while i < INVALID_CMDS.len() {
                let cmd = &INVALID_CMDS[i];
                out[i] = (remove_slash(cmd.0), cmd.1);
                i += 1;
            }
            out
        };

        #[test]
        fn valid_query() -> Result<(), String> {
            for cmd in VALID_CMDS {
                let (addr, _) = cmd;
                let msg = OscMessage::from(*addr);
                let result = X32OscMessage::new(msg)?;
                assert_eq!(result.message.addr, *addr)
            }
            Ok(())
        }

        #[test]
        fn invalid_query() -> Result<(), String> {
            // Invalid addresses
            for cmd in INVALID_CMDS {
                let (addr, _) = cmd;
                let msg = OscMessage::from(*addr);
                let result = X32OscMessage::new(msg);
                assert!(result.is_err());
            }

            // Shouldn't work if there is no / in front
            for cmd in VALID_NO_SLASH_CMDS[0..5].iter() {
                let (addr, _) = cmd;
                let msg = OscMessage::from(*addr);
                let result = X32OscMessage::new(msg);
                assert!(result.is_err());
            }
            Ok(())
        }

        #[test]
        fn valid_node_query() -> Result<(), String> {
            for cmd in VALID_NO_SLASH_CMDS {
                let (addr, _) = cmd;
                let msg = OscMessage {
                    addr: String::from("/node"),
                    args: vec![OscType::String(String::from(addr))],
                };
                let result = X32OscMessage::new(msg)?;
                assert_eq!(result.message.args.len(), 1);
                let OscType::String(str) = &result.message.args[0] else {
                    return Err(String::from("Expected one String Argument"));
                };
                assert_eq!(str, addr)
            }
            Ok(())
        }

        #[test]
        fn invalid_node_query() -> Result<(), String> {
            //TODO: Implement This!
            assert!(false);
            Ok(())
        }

        #[test]
        fn valid_node_cmd() -> Result<(), String> {
            //TODO: Implement This!
            assert!(false);
            Ok(())
        }

        #[test]
        fn valid_cmd() -> Result<(), String> {
            //TODO: Implement This!
            assert!(false);
            Ok(())
        }

        #[test]
        fn invalid_node_cmd() -> Result<(), String> {
            //TODO: Implement This!
            assert!(false);
            Ok(())
        }

        #[test]
        fn invalid_cmd() -> Result<(), String> {
            //TODO: Implement This!
            assert!(false);
            Ok(())
        }

        #[test]
        fn special_addrs() -> Result<(), String> {
            let test_addrs: [&str; 3] = [
                "/xinfo",
                "/status",
                "/xremote",
            ];

            for addr in test_addrs {
                let msg = OscMessage::from(addr);

                let result = X32OscMessage::new(msg);
                assert_eq!(result?.message.addr, addr);
            }
            Ok(())
        }
    }
}

