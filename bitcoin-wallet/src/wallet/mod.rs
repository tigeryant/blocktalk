mod database;
mod interface;
mod notification;
mod transaction;
mod types;
mod config;

pub use database::WalletDatabase;
pub use interface::WalletInterface;
// pub use notification::NotificationProcessor;
// pub use transaction::{TransactionBuilder, TransactionBroadcaster};
pub use types::{TxRecipient, WalletBalance};
pub use config::{WalletConfig, DatabaseConfig};