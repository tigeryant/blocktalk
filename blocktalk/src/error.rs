use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ChainErrorKind {
    BlockNotFound,
    InvalidHeight,
    DeserializationFailed,
    InvalidAncestor,
    InvalidBlockData,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockValidationErrorKind {
    InvalidFormat,
    InvalidHash,
    InvalidMerkleRoot,
    InvalidTransaction,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockTalkError {
    Connection(String),
    Io(String),
    BlockValidation {
        kind: BlockValidationErrorKind,
        message: String,
    },
    Node {
        message: String,
        code: i32,
    },
    Chain {
        kind: ChainErrorKind,
        message: String,
    },
}

impl BlockTalkError {
    pub fn node_error(message: String, code: i32) -> Self {
        BlockTalkError::Node { message, code }
    }

    pub fn validation_error(kind: BlockValidationErrorKind, message: String) -> Self {
        BlockTalkError::BlockValidation { kind, message }
    }

    pub fn chain_error(kind: ChainErrorKind, message: String) -> Self {
        BlockTalkError::Chain { kind, message }
    }
}

impl fmt::Display for BlockTalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockTalkError::Connection(e) => write!(f, "Connection error: {}", e),
            BlockTalkError::Io(e) => write!(f, "IO error: {}", e),
            BlockTalkError::BlockValidation { kind, message } => {
                write!(f, "Block validation error ({:?}): {}", kind, message)
            }
            BlockTalkError::Node { message, code } => {
                write!(f, "Node error ({}): {}", code, message)
            }
            BlockTalkError::Chain { kind, message } => {
                write!(f, "Chain error ({:?}): {}", kind, message)
            }
        }
    }
}

impl Error for BlockTalkError {}

impl From<capnp::Error> for BlockTalkError {
    fn from(error: capnp::Error) -> Self {
        BlockTalkError::Connection(error.to_string())
    }
}

impl From<std::io::Error> for BlockTalkError {
    fn from(error: std::io::Error) -> Self {
        BlockTalkError::Io(error.to_string())
    }
}
