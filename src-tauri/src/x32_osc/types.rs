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

/// Re-packages an OscMessage to ensure that the connection can use it properly.
pub struct X32OscMessage {
    message: OscMessage,
}

impl X32OscMessage {
    /// Puts the input OscMessage into an X32OscMessage after packaging any message with args (assumed to be a set command rather than a request) as
    /// a node set command with path "/". This ensures it will be acknowledged by the other side of the connection
    pub fn new(msg: OscMessage) -> CommandResult<X32OscMessage> {
        match msg.addr.as_str() {
            // Handle special message types by throwing away any arguments and returning a wrapped message
            "/xremote" => Ok(X32OscMessage {message: OscMessage::from(String::from("/xremote"))}),
            "/status" => Ok(X32OscMessage {message: OscMessage::from(String::from("/status"))}),
            "/xinfo" => Ok(X32OscMessage {message: OscMessage::from(String::from("/xinfo"))}),
            // For node queries, unwrap to an address and check that no args are present before validating
            "/node" => {
                if msg.args.len() == 1 && let OscType::String(node_query) = msg.args[0].clone() {
                    if let Some(_) = node_query.find(" ") {
                        Err(CommandError::Parse(String::from("Node query's address should not have any spaces")))
                    } else if validate_msg(&node_query, &Vec::new(), false, true) {
                        Ok(X32OscMessage {
                            message: msg,
                        })
                    } else {
                        Err(CommandError::Parse(String::from("Invalid node to query for IO operations")))
                    }
                } else {
                    Err(CommandError::Parse(String::from("Node query message should only have one String as an argument")))
                }
            },
            // For node commands, check that at least one arg exists, unwrap to an address and arg vector, and validate
            "/" => {
                if msg.args.len() == 1 && let OscType::String(node_cmd) = msg.args[0].clone() {
                    if let Some((addr, arg_str)) = node_cmd.split_once(" ") {
                        let arg_vec = unpack_osc_args(&arg_str);
                        if validate_msg(&String::from(addr), &arg_vec, true, true) {
                            Ok(X32OscMessage {
                                message: msg,
                            })
                        } else {
                            Err(CommandError::Parse(String::from("Invalid node address / args for IO operations")))
                        }
                    } else {
                        Err(CommandError::Parse(String::from("Node command must have arguments")))
                    }
                } else {
                    Err(CommandError::Parse(String::from("Node query message should only have one String as an argument")))
                }
            },
            // Everything else should be a non-node style message
            addr => {
                if addr.chars().collect::<Vec<char>>()[0] == '/' && let Some((_, path)) = addr.split_once("/") {
                    // Handle as a command and convert to node style if args are present
                    if msg.args.len() >= 1 {
                        if validate_msg(&String::from(path), &msg.args, true, false) {
                            make_node_cmd(msg)
                        } else {
                            Err(CommandError::Parse(String::from("Invalid single command path / args for IO operations")))
                        }
                    // Only check address for query if no args present
                    } else {
                        if validate_msg(&String::from(path), &msg.args, false, false) {
                            Ok(X32OscMessage {
                                message: msg,
                            })
                        } else {
                            Err(CommandError::Parse(String::from("Invalid single query path for IO operations")))
                        }
                    }
                } else {
                    Err(CommandError::Parse(String::from("Regular command address should start with a '/'")))
                }
            }
        }
    }

    pub fn get_message(&self) -> OscMessage {
        self.message.clone()
    }
}

fn unpack_osc_args(arg_str: &str) -> Vec<OscType> {
    let mut args: Vec<OscType> = Vec::new();

    for arg in arg_str.split(" ") {
        if let Ok(i) = arg.parse::<i32>() {
            args.push(OscType::Int(i));
        } else if let Ok(f) = arg.parse::<f32>() {
            args.push(OscType::Float(f))
        } else {
            args.push(OscType::String(arg.to_string()))
        }
    }

    args
}

fn validate_msg(addr: &String, args: &Vec<OscType>, check_args: bool, allow_node_addrs: bool) -> bool {
    let input_opt_reg = [
        "AN1-8", "AN9-16", "AN17-24", "AN25-32",
        "A1-8", "A9-16", "A17-24", "A25-32", "A33-40", "A41-48",
        "B1-8", "B9-16", "B17-24", "B25-32", "B33-40", "B41-48",
        "CARD1-8", "CARD9-16", "CARD17-24", "CARD25-32",
        "UIN1-8", "UIN9-16", "UIN17-24", "UIN25-32",
    ];
    let input_opt_aux = [
        "AUX1-4",
        "AN1-2", "AN1-4", "AN1-6",
        "A1-2", "A1-4", "A1-6",
        "B1-2", "B1-4", "B1-6",
        "CARD1-2", "CARD1-4", "CARD1-6",
        "UIN1-2", "UIN1-4", "UIN1-6"
    ];
    let output_opt_reg = [
        "AN1-8", "AN9-16", "AN17-24", "AN25-32",
        "A1-8", "A9-16", "A17-24", "A25-32", "A33-40", "A41-48",
        "B1-8", "B9-16", "B17-24", "B25-32", "B33-40", "B41-48",
        "CARD1-8", "CARD9-16", "CARD17-24", "CARD25-32",
        "OUT1-8", "OUT9-16",
        "P161-8", "P169-16",
        "AUX1-6/Mon", "AuxIN1-6/TB",
        "UOUT1-8", "UOUT9-16", "UOUT17-24", "UOUT25-32", "UOUT33-40", "UOUT41-48",
        "UIN1-8", "UIN9-16", "UIN17-24", "UIN25-32",
    ];

    let addr_tokens: Vec<&str> = addr.split("/").collect();
    let num_tokens = addr_tokens.len();

    match addr_tokens.get(0) {
        Some(&"config") => {
            match addr_tokens.get(1) {
                Some(&"userrout") => {
                    match addr_tokens.get(2) {
                        Some(&"out") => {
                            match addr_tokens.get(3) {
                                Some(val) if num_tokens == 4 => {
                                    let out_range = (1 ..= 48).map(|x| two_digit_string(x)).collect::<Vec<String>>();
                                    if out_range.contains(&val.to_string()) {
                                        !check_args || args.len() == 1 && int_in_range(args.get(0), 0, 208)
                                    } else {false}
                                },
                                Some(_) => {false},
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 47 {
                                                    return false;
                                                } else if !int_in_range(Some(arg), 0, 208) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(&"in") => {
                            match addr_tokens.get(3) {
                                Some(val) if num_tokens == 4 => {
                                    let in_range = (1 ..= 32).map(|x| two_digit_string(x)).collect::<Vec<String>>();
                                    if in_range.contains(&val.to_string()) {
                                        !check_args || args.len() == 1 && int_in_range(args.get(0), 0, 168)
                                    } else {false}
                                },
                                Some(_) => {false}
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 31 {
                                                    return false;
                                                } else if !int_in_range(Some(arg), 0, 168) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(_) => {false},
                        None => {false},
                    }
                },
                Some(&"routing") => {
                    match addr_tokens.get(2) {
                        Some(&"IN") | Some(&"PLAY") => {
                            match addr_tokens.get(3) {
                                Some(val) if ["1-8", "9-16", "17-24", "25-32"].contains(val) && num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 23) || enum_in_list(args.get(0), &input_opt_reg)))
                                },
                                Some(&"AUX") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 15) || enum_in_list(args.get(0), &input_opt_aux)))
                                }
                                Some(_) => {false},
                                // Check that node type is a query of this address or has the right number / type of args
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 32 {
                                                    return false;
                                                } else if idx == 32 && !(int_in_range(Some(arg), 0, 15) || enum_in_list(Some(arg), &input_opt_aux)) {
                                                    return false;
                                                } else if !(int_in_range(Some(arg), 0, 23) || enum_in_list(Some(arg), &input_opt_reg)) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(&"AES50A") | Some(&"AES50B") => {
                            match addr_tokens.get(3) {
                                Some(val) if ["1-8", "9-16", "17-24", "25-32", "33-40", "41-48"].contains(val) && num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 35) || enum_in_list(args.get(0), &output_opt_reg)))
                                },
                                Some(_) => {false},
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 5 {
                                                    return false;
                                                } else if !(int_in_range(Some(arg), 0, 35) || enum_in_list(Some(arg), &output_opt_reg)) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(&"CARD") => {
                            match addr_tokens.get(3) {
                                Some(val) if ["1-8", "9-16", "17-24", "25-32"].contains(val) && num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 35) || enum_in_list(args.get(0), &output_opt_reg)))
                                },
                                Some(_) => {false},
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 3 {
                                                    return false;
                                                } else if !(int_in_range(Some(arg), 0, 35) || enum_in_list(Some(arg), &output_opt_reg)) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(&"OUT") => {
                            let valid_opt_first_half = [
                                "AN1-4", "AN9-12", "AN17-20", "AN25-28",
                                "A1-4", "A9-12", "A17-20", "A25-28", "A33-36", "A41-44",
                                "B1-4", "B9-12", "B17-20", "B25-28", "B33-36", "B41-44",
                                "CARD1-4", "CARD9-12", "CARD17-20", "CARD25-28",
                                "OUT1-4", "OUT9-12",
                                "P161-4", "P169-12",
                                "AUX/CR", "AUX/TB",
                                "UOUT1-4", "UOUT9-12", "UOUT17-20", "UOUT25-28", "UOUT33-36", "UOUT41-44",
                                "UIN1-4", "UIN9-12", "UIN17-20", "UIN25-28",
                            ];

                            let valid_opt_sec_half = [
                                "AN5-8", "AN13-16", "AN21-24", "AN29-32",
                                "A5-8", "A13-16", "A21-24", "A29-32", "A37-40", "A45-48",
                                "B5-8", "B13-16", "B21-24", "B29-32", "B37-40", "B45-48",
                                "CARD5-8", "CARD13-16", "CARD21-24", "CARD29-32",
                                "OUT5-8", "OUT13-16",
                                "P165-8", "P1613-16",
                                "AUX/CR", "AUX/TB",
                                "UOUT5-8", "UOUT13-16", "UOUT21-24", "UOUT29-32", "UOUT37-40", "UOUT45-48",
                                "UIN5-8", "UIN13-16", "UIN21-24", "UIN29-32",
                            ];

                            match addr_tokens.get(3) {
                                Some(val) if ["1-4", "9-12"].contains(val) && num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 35) || enum_in_list(args.get(0), &valid_opt_first_half)))
                                },
                                Some(val) if ["5-8", "13-16"].contains(val) && num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 35) || enum_in_list(args.get(0), &valid_opt_sec_half)))
                                },
                                Some(_) => {false},
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 3 {
                                                    return false;
                                                } else if idx % 2 == 0 && !(int_in_range(Some(arg), 0, 35) || enum_in_list(Some(arg), &valid_opt_first_half)) {
                                                    return false;
                                                } else if !(int_in_range(Some(arg), 0, 35) || enum_in_list(Some(arg), &valid_opt_sec_half)) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(_) => {false},
                        None => {false},
                    }
                },
                Some(_) => {false},
                None => {false},
            }
        },
        Some(&"outputs") => {
            let valid_tap_points = [
                "IN/LC", "IN/LC+M", "<-EQ", "<-EQ+M", "EQ->", "EQ->+M", "PRE", "PRE+M", "POST",
            ];
            let valid_toggle = ["OFF", "ON"];
            match addr_tokens.get(1) {
                Some(&"main") => {
                    let main_range = (1 ..= 16).map(|x| two_digit_string(x)).collect::<Vec<String>>();
                    match addr_tokens.get(2) {
                        Some(val) if main_range.contains(&val.to_string()) => {
                            match addr_tokens.get(3) {
                                Some(&"src") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && int_in_range(args.get(0), 0, 76))
                                },
                                Some(&"pos") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 8) || enum_in_list(args.get(0), &valid_tap_points)))
                                },
                                Some(&"invert") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 1) || enum_in_list(args.get(0), &valid_toggle)))
                                },
                                Some(&"delay") => {
                                    match addr_tokens.get(4) {
                                        Some(&"on") if num_tokens == 5 => {
                                            !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 1) || enum_in_list(args.get(0), &valid_toggle)))
                                        },
                                        Some(&"time") if num_tokens == 5 => {
                                            !check_args || (args.len() == 1 && is_x32_float(args.get(0), 0.300, 500.000, 0.100))
                                        },
                                        Some(_) => {false},
                                        None => {
                                            if allow_node_addrs {
                                                if check_args {
                                                    for (idx, arg) in args.iter().enumerate() {
                                                        if idx > 1 {
                                                            return false;
                                                        } else if idx == 0 && !(int_in_range(Some(arg), 0, 1) || enum_in_list(Some(arg), &valid_toggle)) {
                                                            return false;
                                                        } else if !is_x32_float(Some(arg), 0.300, 500.000, 0.100) {
                                                            return false;
                                                        }
                                                    }
                                                    true
                                                } else {true}
                                            } else {false}
                                        },
                                    }
                                },
                                Some(_) => {false},
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 4 {
                                                    return false;
                                                } else if idx == 0 && !int_in_range(Some(arg), 0, 76) {
                                                    return false;
                                                } else if idx == 1 && !(int_in_range(Some(arg), 0, 8) || enum_in_list(Some(arg), &valid_tap_points)) {
                                                    return false;
                                                } else if idx == 2 && !(int_in_range(Some(arg), 0, 1) || enum_in_list(Some(arg), &valid_toggle)) {
                                                    return false;
                                                } else if idx == 3 && !(int_in_range(Some(arg), 0, 1) || enum_in_list(Some(arg), &valid_toggle)) {
                                                    return false;
                                                } else if idx == 4 && !is_x32_float(Some(arg), 0.300, 500.000, 0.100) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(_) => {false},
                        None => {false},
                    }
                },
                Some(&"aux") => {
                    let aux_range = (1 ..= 6).map(|x| two_digit_string(x)).collect::<Vec<String>>();
                    match addr_tokens.get(2) {
                        Some(val) if aux_range.contains(&val.to_string()) => {
                            match addr_tokens.get(3) {
                                Some(&"src") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && int_in_range(args.get(0), 0, 76))
                                },
                                Some(&"pos") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 8) || enum_in_list(args.get(0), &valid_tap_points)))
                                },
                                Some(&"invert") if num_tokens == 4 => {
                                    !check_args || (args.len() == 1 && (int_in_range(args.get(0), 0, 1) || enum_in_list(args.get(0), &valid_toggle)))
                                },
                                Some(_) => {false},
                                None => {
                                    if allow_node_addrs {
                                        if check_args {
                                            for (idx, arg) in args.iter().enumerate() {
                                                if idx > 2 {
                                                    return false;
                                                } else if idx == 0 && !int_in_range(Some(arg), 0, 76) {
                                                    return false;
                                                } else if idx == 1 && !(int_in_range(Some(arg), 0, 8) || enum_in_list(Some(arg), &valid_tap_points)) {
                                                    return false;
                                                } else if idx == 2 && !(int_in_range(Some(arg), 0, 1) || enum_in_list(Some(arg), &valid_toggle)) {
                                                    return false;
                                                }
                                            }
                                            true
                                        } else {true}
                                    } else {false}
                                },
                            }
                        },
                        Some(_) => {false},
                        None => {false},
                    }
                },
                Some(_) => {false},
                None => {false},
            }
        },
        Some(_) => {false},
        None => {false},
    }
}

fn int_in_range(arg: Option<&OscType>, min: i32, max: i32) -> bool {
    if let Some(OscType::Int(int)) = arg {
        if int <= &max && int >= &min {
            true
        } else {false}
    } else {
        false
    }
}

fn enum_in_list(arg: Option<&OscType>, list: &[&str]) -> bool {
    if let Some(OscType::String(s)) = arg {
        list.contains(&s.as_str())
    } else {
        false
    }
}

fn is_x32_float(arg: Option<&OscType>, min: f32, max: f32, step_size: f32) -> bool {
    if let Some(OscType::Float(f)) = arg {
        f >= &min && f <= &max && f % &step_size < 1e-12
    } else {
        false
    }
}

/// Converts a standard X32 command into a node style command so that it will be acknowledged when
/// sent to the X32 console rather than being silently accepted.

fn make_node_cmd(msg: OscMessage) -> Result<X32OscMessage, CommandError> {
    if msg.args.len() >= 1 {
        let mut node_string = String::from("");

        node_string.push_str(&msg.addr[1..]);

        for arg in msg.args {
            match arg {
                OscType::Int(i) => node_string.push_str(&(" ".to_string() + &i.to_string())),
                OscType::Float(f) => node_string.push_str(&(" ".to_string() + &f.to_string())),
                OscType::String(s) => node_string.push_str(&(" ".to_string() + &s)),
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
}

/// Returns the string representing an integer as two digits with
/// a leading 0 if needed. Used to check command or query paths more easily
/// by mapping a range of integers.
fn two_digit_string(int: i32) -> String {
    format!("{:02}", int)
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
                "/outputs/aux/06/pos",
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
                ("/config/userrout/in/01", vec![OscType::Int(165)], "config/userrout/in/01 165"),
                ("/config/userrout/in/05", vec![OscType::Int(168)], "config/userrout/in/05 168"),
                ("/config/routing/IN/1-8", vec![OscType::Int(2)], "config/routing/IN/1-8 2"),
                ("/config/routing/IN/9-16", vec![OscType::Int(23)], "config/routing/IN/9-16 23"),
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
                ("/outputs/aux/06/pos", vec![OscType::Int(7)], "outputs/aux/06/pos 7"),
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
                "config/userrout/in/01 0",
                "config/userrout/in/05 168",
                "config/routing/IN/1-8 0",
                "config/routing/IN/9-16 23",
                "outputs/aux/06/pos 7",
                // Full node set commands
                "config/userrout/out 0 1 2 3 4 5 6 7 8 9",
                "config/userrout/in 0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31",
                "config/routing/IN 0 0 0 0",
                "config/routing/AES50A 0 1 2 3 4 5",
                "config/routing/AES50B 0 1 2",
                "config/routing/CARD 0 1 4 8",
                "config/routing/OUT 1 2 3 5",
                "outputs/aux/05 0 6",
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
                "outputs/main/01 56 10",
                // Too many args for address:
                "config/userrout/out 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47 48 49",
                "config/routing/OUT 32 31 35 34 1",
                "outputs/main/01 50 7 0 1 0.300 5",
                // Invalid Node:
                "outputs/main 1",
                "config/userrout 1",
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

