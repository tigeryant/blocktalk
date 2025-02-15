use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum BlockTalkError {
    ConnectionError(capnp::Error),
    IoError(std::io::Error),
    InvalidBlockData,
    NodeError(String),
}

impl fmt::Display for BlockTalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockTalkError::ConnectionError(e) => write!(f, "IPC connection error: {}", e),
            BlockTalkError::IoError(e) => write!(f, "IO error: {}", e),
            BlockTalkError::InvalidBlockData => write!(f, "Invalid block data"),
            BlockTalkError::NodeError(s) => write!(f, "Node error: {}", s),
        }
    }
}

impl Error for BlockTalkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BlockTalkError::ConnectionError(e) => Some(e),
            BlockTalkError::IoError(e) => Some(e),
            BlockTalkError::InvalidBlockData => None,
            BlockTalkError::NodeError(_) => None,
        }
    }
}

impl From<capnp::Error> for BlockTalkError {
    fn from(error: capnp::Error) -> Self {
        BlockTalkError::ConnectionError(error)
    }
}

impl From<std::io::Error> for BlockTalkError {
    fn from(error: std::io::Error) -> Self {
        BlockTalkError::IoError(error)
    }
}