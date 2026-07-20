use std::{fmt, io, net};
use std::error::Error;
use std::fmt::Display;
use std::sync::PoisonError;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug)]
pub enum CommandError {
    OSC(rosc::OscError),
    IO(io::Error),
    AddrParse(net::AddrParseError),
    OtherErr(Box<dyn Error>),
    Parse(String),
    InvalidOp(String),
    Mutex,
}

pub type CommandResult<T> = Result<T, CommandError>;

impl Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            CommandError::OSC(err) => format!("OSC error while handling command: {}", err),
            CommandError::IO(err) => format!("IO error while handling command: {}", err),
            CommandError::AddrParse(err) => format!("Failed to parse IP Address: {}", err),
            CommandError::OtherErr(err) => format!("Error handling command: {}", err),
            CommandError::Parse(err) => format!("Parse error while handling command: {}", err),
            CommandError::InvalidOp(err) => format!("Command was invalid: {}", err),
            CommandError::Mutex => String::from("Failed to lock mutex for command"),
        };
        write!(f, "{}", str)
    }
}

impl From<CommandError> for String {
    fn from(value: CommandError) -> Self {
        value.to_string()
    }
}

impl Error for CommandError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CommandError::OSC(e) => Some(e),
            CommandError::IO(e) => Some(e),
            CommandError::AddrParse(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CommandError {
    fn from(err: io::Error) -> Self {
        CommandError::IO(err)
    }
}

impl From<rosc::OscError> for CommandError {
    fn from(err: rosc::OscError) -> Self {
        CommandError::OSC(err)
    }
}

impl From<net::AddrParseError> for CommandError {
    fn from(err: net::AddrParseError) -> Self {
        CommandError::AddrParse(err)
    }
}

impl From<RecvError> for CommandError {
    fn from(err: RecvError) -> Self {CommandError::OtherErr(Box::new(err))}
}

impl<T> From<PoisonError<T>> for CommandError {
    fn from(_: PoisonError<T>) -> Self {
        Self::Mutex
    }
}