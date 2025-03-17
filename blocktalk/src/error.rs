use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ChainErrorKind {
    BlockNotFound,
    InvalidHeight,
    DeserializationFailed,
    InvalidAncestor,
    InvalidBlockData,
    Other(String),
}

#[derive(Debug)]
pub enum BlockValidationErrorKind {
    InvalidFormat,
    InvalidHash,
    InvalidMerkleRoot,
    InvalidTransaction,
    Other(String),
}

#[derive(Debug)]
pub enum BlockTalkError {
    Connection(capnp::Error),
    Io(std::io::Error),
    BlockValidation {
        kind: BlockValidationErrorKind,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
    Node {
        message: String,
        code: i32,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
    Chain {
        kind: ChainErrorKind,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}

impl BlockTalkError {
    pub fn node_error<T: Into<String>>(message: T, code: i32) -> Self {
        Self::Node {
            message: message.into(),
            code,
            source: None,
        }
    }

    pub fn validation_error(kind: BlockValidationErrorKind) -> Self {
        Self::BlockValidation { kind, source: None }
    }

    pub fn chain_error(kind: ChainErrorKind) -> Self {
        Self::Chain { kind, source: None }
    }

    pub fn with_source(self, source: impl Error + Send + Sync + 'static) -> Self {
        match self {
            Self::Node { message, code, .. } => Self::Node {
                message,
                code,
                source: Some(Box::new(source)),
            },
            Self::BlockValidation { kind, .. } => Self::BlockValidation {
                kind,
                source: Some(Box::new(source)),
            },
            Self::Chain { kind, .. } => Self::Chain {
                kind,
                source: Some(Box::new(source)),
            },
            _ => self,
        }
    }
}

impl fmt::Display for BlockTalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(e) => write!(f, "IPC connection error: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::BlockValidation { kind, source } => {
                write!(f, "Block validation error: {:?}", kind)?;
                if let Some(src) = source {
                    write!(f, " (Caused by: {})", src)?;
                }
                Ok(())
            }
            Self::Node {
                message,
                code,
                source,
            } => {
                write!(f, "Node error (code {}): {}", code, message)?;
                if let Some(src) = source {
                    write!(f, " (Caused by: {})", src)?;
                }
                Ok(())
            }
            Self::Chain { kind, source } => {
                write!(f, "Chain error: {:?}", kind)?;
                if let Some(src) = source {
                    write!(f, " (Caused by: {})", src)?;
                }
                Ok(())
            }
        }
    }
}

impl Error for BlockTalkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connection(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::BlockValidation { source, .. } => {
                source.as_ref().map(|e| e.as_ref() as &dyn Error)
            }
            Self::Node { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
            Self::Chain { source, .. } => source.as_ref().map(|e| e.as_ref() as &dyn Error),
        }
    }
}

impl From<capnp::Error> for BlockTalkError {
    fn from(error: capnp::Error) -> Self {
        Self::Connection(error)
    }
}

impl From<std::io::Error> for BlockTalkError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
