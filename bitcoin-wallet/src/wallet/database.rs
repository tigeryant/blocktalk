// use std::path::{Path, PathBuf};
// use bitcoin::Network;
// use bdk_wallet::{PersistedWallet, Wallet};

// use crate::error::WalletError;
// use super::types::TransactionMetadata;

// pub struct WalletDatabase {
//     /// Path to the SQLite database file
//     db_path: PathBuf,
// }

// impl WalletDatabase {
//     pub fn new(db_path: PathBuf) -> Self {
//         Self { db_path }
//     }
    
//     pub fn path(&self) -> &Path {
//         &self.db_path
//     }
    
//     pub fn open_connection(&self) -> Result<rusqlite::Connection, WalletError> {
//         rusqlite::Connection::open(&self.db_path)
//             .map_err(|e| WalletError::DatabaseError(format!("Failed to open database: {}", e)))
//     }
    
//     pub fn exists(&self) -> bool {
//         self.db_path.exists()
//     }
    
//     pub fn load_wallet(&self, network: Network) -> Result<PersistedWallet<rusqlite::Connection>, WalletError> {
//         let mut conn = self.open_connection()?;
        
//         let persisted = Wallet::load()
//             .check_network(network)
//             .load_wallet(&mut conn)
//             .map_err(|e| WalletError::Generic(format!("Failed to load wallet: {}", e)))?;
        
//         match persisted {
//             Some(persisted_wallet) => Ok(persisted_wallet),
//             None => Err(WalletError::Generic("No wallet found in database".to_string())),
//         }
//     }
    
//     pub fn create_wallet(
//         &self, 
//         external_descriptor: String,
//         internal_descriptor: String,
//         network: Network
//     ) -> Result<PersistedWallet<rusqlite::Connection>, WalletError> {
//         let mut conn = self.open_connection()?;
        
//         let persisted = Wallet::create(external_descriptor, internal_descriptor)
//             .network(network)
//             .create_wallet(&mut conn)
//             .map_err(|e| WalletError::Generic(format!("Failed to create wallet: {}", e)))?;
            
//         Ok(persisted)
//     }
    
//     // pub fn store_tx_metadata(&self, txid: &bitcoin::Txid, metadata: &TransactionMetadata) -> Result<(), WalletError> {
//     //     let mut conn = self.open_connection()?;
        
//     //     // Create table if it doesn't exist
//     //     conn.execute(
//     //         "CREATE TABLE IF NOT EXISTS tx_metadata (
//     //             txid TEXT PRIMARY KEY,
//     //             timestamp INTEGER NOT NULL,
//     //             block_height INTEGER,
//     //             fee_sat INTEGER,
//     //             comment TEXT NOT NULL,
//     //             label TEXT NOT NULL
//     //         )",
//     //         [],
//     //     ).map_err(|e| WalletError::DatabaseError(format!("Failed to create table: {}", e)))?;
        
//     //     // Insert or replace metadata
//     //     conn.execute(
//     //         "INSERT OR REPLACE INTO tx_metadata 
//     //         (txid, timestamp, block_height, fee_sat, comment, label)
//     //         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
//     //         rusqlite::params![
//     //             txid.to_string(),
//     //             metadata.timestamp as i64,
//     //             metadata.block_height.map(|h| h as i64),
//     //             metadata.fee.map(|f| f.to_sat() as i64),
//     //             metadata.comment,
//     //             metadata.label,
//     //         ],
//     //     ).map_err(|e| WalletError::DatabaseError(format!("Failed to store metadata: {}", e)))?;
        
//     //     Ok(())
//     // }
// }