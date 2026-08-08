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

        #[test]
        fn valid_query() -> Result<(), String> {
            let valid = [
                "/config/userrout/out/01",
                "/config/userrout/out/02",
                "/config/userrout/in/01",
                "/config/userrout/in/05",
                "/config/routing/IN/1-8",
                "/config/routing/IN/9-16",
                "/config/routing/IN/AUX",
                "/config/routing/AES50A/1-8",
                "/config/routing/AES50A/9-16",
                "/config/routing/AES50B/1-8",
                "/config/routing/AES50B/9-16",
                "/config/routing/CARD/1-8",
                "/config/routing/CARD/9-16",
                "/config/routing/OUT/1-4",
                "/config/routing/OUT/5-8",
                "/outputs/main/01/src",
                "/outputs/main/16/pos",
                "/outputs/aux/01/src",
                "/outputs/aux/16/pos",
            ];

            for addr in valid {
                let msg = OscMessage::from(addr);
                let result = X32OscMessage::new(msg)?;
                assert_eq!(result.message.addr, *addr)
            }
            Ok(())
        }

        #[test]
        fn invalid_query() -> Result<(), String> {
            let invalid = [
                // Incorrect numbers or locations:
                "/config/userrout/out/00",
                "/config/userrout/out/50",
                "/config/userrout/in/00",
                "/config/userrout/in/35",
                "/config/routing/IN/1-9",
                "/config/routing/IN/8-17",
                "/config/routing/IN/AUXS",
                "/config/routing/AES50C/1-8",
                "/config/routing/AES50C6",
                "/config/routing/AES50B/1-9",
                "/config/routing/AES50B/8-16",
                "/config/routing/CARD/1-10",
                "/config/routing/CARD/9-18",
                "/config/routing/OUT/1-8",
                "/config/routing/OUT/9-16",
                "/config/routing/OUT/",
                "/config/routing/OUT/1-4/",
                "/outputs/main/00/src",
                "/outputs/main/18/pos",
                "/outputs/aux/01/src/",
                "/outputs/aux/18/pos",
                // Missing leading slash:
                "config/routing/AES50B/1-8",
                "config/routing/AES50B/9-16",
                // Short path:
                "/config/routing/IN",
                // Path unrelated to this app:
                "/ch/01/gate/on",
            ];

            for addr in invalid {
                let msg = OscMessage::from(addr);
                let result = X32OscMessage::new(msg);
                assert!(result.is_err());
            }
            Ok(())
        }

        #[test]
        fn valid_node_query() -> Result<(), String> {
            let valid_node: &[&str] = [
                // Full addrs:
                "config/userrout/out/01",
                "config/userrout/out/02",
                "config/userrout/in/01",
                "config/userrout/in/05",
                // Node addr:
                "config/userrout/in",
                "config/routing/OUT"
            ].as_slice();

            for &addr in valid_node {
                let msg = OscMessage {
                    addr: String::from("/node"),
                    args: vec![OscType::String(String::from(addr))],
                };
                let result = X32OscMessage::new(msg)?;
                assert_eq!(result.message.addr, "/node");
                assert_eq!(result.message.args, vec![OscType::String(String::from(addr))]);
            }
            Ok(())
        }

        #[test]
        fn invalid_node_query() -> Result<(), String> {
            let invalid_node: &[&str] = [
                "/config/userrout/in/00",
                "config/userrout/in/35",
                "/config/routing/IN/1-9/",
                "config/routing/IN/8-17",
                "config/routing/IN/AUXS",
                "config/routing",
                "config/userrout/",
            ].as_slice();

            for &addr in invalid_node {
                let msg = OscMessage {
                    addr: String::from("/node"),
                    args: vec![OscType::String(String::from(addr))],
                };
                let result = X32OscMessage::new(msg);
                assert!(result.is_err());
            }
            Ok(())
        }

        #[test]
        fn valid_cmd() -> Result<(), String> {
            let valid_cmds = [
                ("/config/userrout/out/01", vec![OscType::Int(0)], "config/userrout/out/01 0"),
                ("/config/userrout/out/02", vec![OscType::Int(184)], "config/userrout/out/02 184"),
                ("/config/userrout/in/01", vec![OscType::Int(184)], "config/userrout/in/01 184"),
                ("/config/userrout/in/05", vec![OscType::Int(184)], "config/userrout/in/05 184"),
                ("/config/routing/IN/1-8", vec![OscType::Int(184)], "config/routing/IN/1-8 184"),
                ("/config/routing/IN/9-16", vec![OscType::Int(184)], "config/routing/IN/9-16 184"),
                ("/config/routing/IN/AUX", vec![OscType::Int(10)], "config/routing/IN/AUX 10"),
                ("/config/routing/AES50A/1-8", vec![OscType::Int(0)], "config/routing/AES50A/1-8 0"),
                ("/config/routing/AES50A/9-16", vec![OscType::Int(35)], "config/routing/AES50A/9-16 35"),
                ("/config/routing/AES50B/1-8", vec![OscType::Int(0)], "config/routing/AES50B/1-8 0"),
                ("/config/routing/AES50B/9-16", vec![OscType::Int(35)], "config/routing/AES50B/9-16 35"),
                ("/config/routing/CARD/1-8", vec![OscType::Int(0)], "config/routing/CARD/1-8 0"),
                ("/config/routing/CARD/9-16", vec![OscType::Int(35)], "config/routing/CARD/9-16 35"),
                ("/config/routing/OUT/1-4", vec![OscType::Int(0)], "config/routing/OUT/1-4 0"),
                ("/config/routing/OUT/5-8", vec![OscType::Int(35)], "config/routing/OUT/5-8 35"),
                ("/outputs/main/01/src", vec![OscType::Int(10)], "outputs/main/01/src 10"),
                ("/outputs/main/16/pos", vec![OscType::Int(8)], "outputs/main/16/pos 8"),
                ("/outputs/aux/01/src", vec![OscType::Int(10)], "outputs/aux/01/src 10"),
                ("/outputs/aux/16/pos", vec![OscType::Int(7)], "outputs/aux/16/pos 7"),
            ];

            for cmd in valid_cmds {
                let (addr, args, node_str) = cmd;
                let msg = OscMessage {
                    addr: String::from(addr),
                    args,
                };
                let result = X32OscMessage::new(msg)?;
                assert_eq!(result.message.addr, "/");
                assert_eq!(result.message.args.len(), 1);
                let OscType::String(ref str) = result.message.args[0] else {
                    return Err(String::from("Expected one string argument"));
                };
                assert_eq!(str, node_str);
            }
            Ok(())
        }

        #[test]
        fn invalid_cmd() -> Result<(), String> {
            let invalid_cmds = [
                // Incorrect numbers or locations:
                ("/config/userrout/out/00", vec![OscType::Int(0)]),
                ("/config/userrout/out/50", vec![OscType::Int(0)]),
                ("/config/userrout/in/00", vec![OscType::Int(0)]),
                ("/config/userrout/in/35", vec![OscType::Int(0)]),
                ("/config/routing/IN/1-9", vec![OscType::Int(0)]),
                ("/config/routing/IN/8-17", vec![OscType::Int(0)]),
                ("/config/routing/IN/AUXS", vec![OscType::Int(0)]),
                ("/config/routing/AES50C/1-8", vec![OscType::Int(0)]),
                ("/config/routing/AES50C6", vec![OscType::Int(0)]),
                ("/config/routing/AES50B/1-9", vec![OscType::Int(0)]),
                ("/config/routing/AES50B/8-16", vec![OscType::Int(0)]),
                ("/config/routing/CARD/1-10", vec![OscType::Int(0)]),
                ("/config/routing/CARD/9-18", vec![OscType::Int(0)]),
                ("/config/routing/OUT/1-8", vec![OscType::Int(0)]),
                ("/config/routing/OUT/9-16", vec![OscType::Int(0)]),
                ("/config/routing/OUT/", vec![OscType::Int(0)]),
                ("/config/routing/OUT/1-4/", vec![OscType::Int(0)]),
                ("/outputs/main/00/src", vec![OscType::Int(0)]),
                ("/outputs/main/18/pos", vec![OscType::Int(0)]),
                ("/outputs/aux/01/src/", vec![OscType::Int(0)]),
                ("/outputs/aux/18/pos", vec![OscType::Int(0)]),
                // Missing leading slash:
                ("config/routing/AES50B/1-8", vec![OscType::Int(0)]),
                ("config/routing/AES50B/9-16", vec![OscType::Int(0)]),
                // Short path:
                ("/config/routing/IN", vec![OscType::Int(0)]),
                // Path unrelated to this app:
                ("/ch/01/gate/on", vec![OscType::Int(0)]),
                // Invalid args for address:
                ("/config/userrout/out/01", vec![OscType::Int(-1)]),
                ("/config/userrout/out/02", vec![OscType::Int(300)]),
                ("/config/routing/OUT/1-4", vec![OscType::Int(-5)]),
                ("/config/routing/OUT/5-8", vec![OscType::Int(50)]),
            ];

            for cmd in invalid_cmds {
                let (addr, args) = cmd;
                let msg = OscMessage {
                    addr: String::from(addr),
                    args,
                };
                let result = X32OscMessage::new(msg);
                assert!(result.is_err());
            }
            Ok(())
        }

        #[test]
        fn valid_node_cmd() -> Result<(), String> {
            let valid_node_cmds = [
                "config/userrout/out/01 0",
                "config/userrout/out/02 184",
                "config/userrout/in/01 184",
                "config/userrout/in/05 184",
                "config/routing/IN/1-8 184",
                "config/routing/IN/9-16 184",
                "outputs/aux/16/pos 7",
                // Full node set commands
                "config/userrout/out 0 1 2 3 4 5 6 7 8 9",
                "config/userrout/in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32",
                "config/routing/IN 0 0 0 0",
                "config/routing/AES50A 0 1 2 3 4 5",
                "config/routing/AES50B 0 1 2",
                "config/routing/CARD 0 1 4 8",
                "config/routing/OUT 1 2 3 5",
                "outputs/aux/05 0 0",
                "outputs/main/14 6 1",
            ];

            for cmd in valid_node_cmds {
                let msg = OscMessage {
                    addr: String::from("/"),
                    args: vec![OscType::String(String::from(cmd))],
                };
                let Ok(result) = X32OscMessage::new(msg) else {
                    return Err(String::from("Path should be a valid command"))
                };
                assert_eq!(result.message.addr, "/");
                assert_eq!(result.message.args.len(), 1);
                let OscType::String(arg) = &result.message.args[0] else {
                    return Err(String::from("Expected node command to have one string"))
                };
                assert_eq!(arg, cmd);
            }
            Ok(())
        }

        #[test]
        fn invalid_node_cmd() -> Result<(), String> {
            let invalid_node_cmds = [
                // Incorrect numbers or locations:
                "config/userrout/out/00",
                "config/userrout/out/50",
                "config/userrout/in/00",
                "config/userrout/in/35",
                "config/routing/IN/1-9",
                "config/routing/IN/8-17",
                "config/routing/IN/AUXS",
                "config/routing/AES50C/1-8",
                "config/routing/AES50C6",
                "config/routing/AES50B/1-9",
                "config/routing/AES50B/8-16",
                "config/routing/CARD/1-10",
                "config/routing/CARD/9-18",
                "config/routing/OUT/1-8",
                "config/routing/OUT/9-16 0",
                "config/routing/OUT/ 0",
                "config/routing/OUT/1-4/ 0",
                "outputs/main/00/src 0",
                "outputs/main/18/pos 0",
                "outputs/aux/01/src/ 0",
                "outputs/aux/18/pos 0",
                // Path unrelated to this app:
                "ch/01/gate/on 0",
                // Invalid args for address:
                "config/userrout/out/01 -1",
                "config/userrout/out/02 300",
                "config/routing/OUT/1-4 -5",
                "config/routing/OUT/5-8 50",
                // TODO: Add cases for more args than node can handle
                //  and eliminate some unneeded cases, generally check that all the
                //  the tests are correct
            ];

            for cmd in invalid_node_cmds {
                let msg = OscMessage {
                    addr: String::from("/"),
                    args: vec![OscType::String(String::from(cmd))],
                };
                let result = X32OscMessage::new(msg);
                assert!(result.is_err());
            }
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

