use bitcoin::Txid;
use std::fmt;
use thiserror::Error;

/// Errors that can occur in the bitcoin-wallet
#[derive(Error, Debug)]
pub enum WalletError {
    /// Communication with node error (via blocktalk)
    #[error("Blocktalk error: {0}")]
    BlocktalkError(#[from] blocktalk::BlockTalkError),

    /// Bitcoin error
    #[error("Bitcoin error: {0}")]
    BitcoinError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    /// RPC error
    #[error("RPC error: {0}")]
    RPCError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Transaction not found
    #[error("Transaction not found: {0}")]
    TransactionNotFound(Txid),

    /// Invalid descriptor
    #[error("Invalid descriptor: {0}")]
    InvalidDescriptor(String),

    /// Key derivation failed
    #[error("Key derivation failed")]
    KeyDerivationFailed,

    /// Unsupported operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Generic error
    #[error("{0}")]
    Generic(String),

    /// Passphrase error
    #[error("Passphrase error: {0}")]
    PassphraseError(String),
    // #[error("BDK error: {0}")]
    // BDKError(#[from] bdk_wallet::error::E),
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
