mod x32_message;

use rosc::OscMessage;
use serde::Serialize;
use std::net::IpAddr;
use tokio::sync::oneshot;

pub use x32_message::*;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    mod x32_console {
        use std::net::IpAddr;
        use std::net::Ipv4Addr;

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
}

