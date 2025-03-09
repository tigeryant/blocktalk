//! Common types used in the wallet module

use bitcoin::{Amount, BlockHash, ScriptBuf, Txid};

/// Transaction recipient for creating transactions
#[derive(Clone)]
pub struct TxRecipient {
    /// Recipient script
    pub script: ScriptBuf,
    
    /// Amount to send
    pub amount: Amount,
}

/// Balance information for the wallet (matches Bitcoin Core format)
#[derive(Debug, Clone, Copy)]
pub struct WalletBalance {
    /// Confirmed balance
    pub confirmed: Amount,
    
    /// Unconfirmed balance (pending)
    pub unconfirmed: Amount,
    
    /// Immature balance (coinbase)
    pub immature: Amount,
    
    /// Total balance
    pub total: Amount,
}

/// Transaction metadata for wallet operations
#[derive(Clone, Debug)]
pub(crate) struct TransactionMetadata {
    /// Time the transaction was received
    pub timestamp: u64,
    
    /// Block the transaction was confirmed in
    pub block_height: Option<u32>,
    
    /// Transaction fee (if sent by this wallet)
    pub fee: Option<Amount>,
    
    /// Transaction comment (for Bitcoin Core compatibility)
    pub comment: String,
    
    /// Label for the transaction
    pub label: String,
}

/// Events that trigger wallet actions
#[derive(Debug)]
pub(crate) enum WalletEvent {
    /// A new block was connected to the chain
    BlockConnected(bitcoin::Block),
    
    /// A block was disconnected from the chain
    BlockDisconnected(BlockHash),
    
    /// A transaction was detected (either in a block or mempool)
    TransactionDetected(bitcoin::Transaction),
    
    /// A request to sync the wallet was received
    SyncRequested,
}

pub struct CreateWalletOptions {
    pub wallet_name: String,
    pub disable_private_keys: bool,
    pub blank: bool,
    pub passphrase: Option<String>,
    pub avoid_reuse: bool,
    pub descriptors: bool,
    pub load_on_startup: bool,
}

impl Default for CreateWalletOptions {
    fn default() -> Self {
        Self {
            wallet_name: String::new(),
            disable_private_keys: false,
            blank: false,
            passphrase: None,
            avoid_reuse: false,
            descriptors: true,
            load_on_startup: false,
        }
    }
}