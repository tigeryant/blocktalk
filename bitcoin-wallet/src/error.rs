use bitcoin::Txid;
use std::fmt;
use thiserror::Error;

/// Errors that can occur in the bitcoin-wallet
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Blocktalk error: {0}")]
    BlocktalkError(#[from] blocktalk::BlockTalkError),

    #[error("Bitcoin error: {0}")]
    BitcoinError(String),

    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("RPC error: {0}")]
    RPCError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Transaction not found: {0}")]
    TransactionNotFound(Txid),

    #[error("Invalid descriptor: {0}")]
    InvalidDescriptor(String),
    
    #[error("{0}")]
    Generic(String),
}

impl From<jsonrpc_core::Error> for WalletError {
    fn from(error: jsonrpc_core::Error) -> Self {
        WalletError::RPCError(error.to_string())
    }
}

impl From<String> for WalletError {
    fn from(error: String) -> Self {
        WalletError::Generic(error)
    }
}
