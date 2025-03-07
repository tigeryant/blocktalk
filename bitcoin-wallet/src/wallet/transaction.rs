// use std::sync::{Arc, RwLock};
// use bitcoin::{Amount, Transaction, Txid, psbt::Psbt};
// use bdk_wallet::Wallet;
// use chrono::Utc;

// use crate::error::WalletError;
// use super::database::WalletDatabase;
// use super::types::{TransactionMetadata, TxRecipient};

// pub struct TransactionBuilder {
//     wallet: Arc<RwLock<Wallet>>,
//     db: WalletDatabase,
// }

// impl TransactionBuilder {
//     pub fn new(wallet: Arc<RwLock<Wallet>>, db: WalletDatabase) -> Self {
//         Self { wallet, db }
//     }
    
//     pub fn create_transaction(
//         &self,
//         recipients: &[TxRecipient],
//         fee_rate: Option<f64>,
//         subtract_fee_from_amount: Option<Vec<usize>>,
//     ) -> Result<(Transaction, Psbt), WalletError> {
//         let mut wallet = self.wallet.write().unwrap();
        
//         let mut conn = self.db.open_connection()?;
//         let mut tx_draft = wallet.build_tx()
//             .map_err(|e| WalletError::BDKError(e))?;
        
//         for (i, recipient) in recipients.iter().enumerate() {
//             let should_subtract_fee = subtract_fee_from_amount
//                 .as_ref()
//                 .map(|indices| indices.contains(&i))
//                 .unwrap_or(false);
            
//             tx_draft.add_recipient(recipient.script.clone(), recipient.amount.to_sat())
//                 .map_err(|e| WalletError::BDKError(e))?;
                
//             if should_subtract_fee {
//                 tx_draft.subtract_fee_from_last()
//                     .map_err(|e| WalletError::BDKError(e))?;
//             }
//         }
        
//         if let Some(rate) = fee_rate {
//             tx_draft.fee_rate(rate)
//                 .map_err(|e| WalletError::BDKError(e))?;
//         }
        
//         let (psbt, _details) = tx_draft.finish()
//             .map_err(|e| WalletError::BDKError(e))?;
        
//         let signed_psbt = wallet.sign(psbt.clone(), &mut conn)
//             .map_err(|e| WalletError::BDKError(e))?;
        
//         let tx = signed_psbt.extract_tx()
//             .map_err(|e| WalletError::BDKError(e))?;
        
//         Ok((tx, signed_psbt))
//     }
    
//     pub fn sign_transaction(&self, psbt: &mut Psbt) -> Result<bool, WalletError> {
//         let mut wallet = self.wallet.write().unwrap();
//         let mut conn = self.db.open_connection()?;
            
//         let finalized = wallet.sign(psbt, SignOptions::default())
//             .map_err(|e| WalletError::BDKError(e))?;
        
//         Ok(finalized)
//     }
// }

// pub struct TransactionBroadcaster {
//     db: WalletDatabase,
//     blocktalk: Arc<blocktalk::BlockTalk>,
// }

// impl TransactionBroadcaster {
//     pub fn new(db: WalletDatabase, blocktalk: Arc<blocktalk::BlockTalk>) -> Self {
//         Self { db, blocktalk }
//     }
    
//     pub async fn broadcast_transaction(&self, tx: &Transaction) -> Result<Txid, WalletError> {
//         let txid = tx.txid();
        
//         // FIXME: add a method to blocktalk to broadcast transactions
//         log::info!("Broadcasting transaction: {}", txid);
//         log::info!("Transaction successfully sent: {}", txid);
        
//         let timestamp = Utc::now().timestamp() as u64;
//         let metadata = TransactionMetadata {
//             timestamp,
//             block_height: None, // Not confirmed yet
//             fee: None, // Fee calculation would be provided by BDK
//             comment: String::new(),
//             label: String::new(),
//         };
        
//         // Store metadata in database
//         self.db.store_tx_metadata(&txid, &metadata)?;
        
//         Ok(txid)
//     }
    
//     pub fn calculate_fee(&self, tx: &Transaction) -> Option<Amount> {
//         None
//     }
// }