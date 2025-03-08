use std::path::Path;
use std::sync::{Arc, RwLock};
use bitcoin::{Address, Block, Network, Transaction, psbt::Psbt};
use bdk_wallet::{KeychainKind, PersistedWallet, rusqlite};
use tokio::sync::mpsc;
use rand::{self, Rng};

use crate::error::WalletError;
use super::database::WalletDatabase;
use super::notification::NotificationProcessor;
// use super::transaction::{TransactionBuilder, TransactionBroadcaster};
use super::types::{TransactionMetadata, WalletBalance, TxRecipient};

pub struct WalletInterface {
    wallet: Arc<RwLock<PersistedWallet<rusqlite::Connection>>>,
    db: WalletDatabase,
    // blocktalk: Arc<blocktalk::BlockTalk>,
    network: Network,
    // tx_builder: TransactionBuilder,
    // tx_broadcaster: TransactionBroadcaster,
    // notification_tx: mpsc::Sender<WalletEvent>,
}

impl WalletInterface {
    pub async fn new(
        wallet_path: &Path,
        node_socket: &str,
        network: Network,
    ) -> Result<Arc<Self>, WalletError> {
        log::info!("Initializing wallet with network: {:?}", network);
        
        // let (tx, rx) = mpsc::channel(100);
        
        // let blocktalk = Arc::new(blocktalk::BlockTalk::init(node_socket).await?);
        // log::info!("Connected to Bitcoin node via blocktalk");
        
        let db_path = wallet_path.with_extension("sqlite3");
        let db = WalletDatabase::new(db_path.to_owned());
        
        let wallet = if db.exists() {
            log::info!("Loading existing wallet from {}", db_path.display());
            db.load_wallet(network)?
        } else {
            log::info!("Creating new wallet at {}", db_path.display());
            let (external_desc, internal_desc) = generate_descriptors(network)?;
            db.create_wallet(external_desc, internal_desc, network)?
        };
        
        let wallet = Arc::new(RwLock::new(wallet));
        
        // let tx_builder = TransactionBuilder::new(wallet.clone(), db.clone());
        // let tx_broadcaster = TransactionBroadcaster::new(db.clone(), blocktalk.clone());
        
        // Create wallet interface
        let wallet_interface = Arc::new(Self {
            wallet,
            db,
            // blocktalk: blocktalk.clone(),
            network,
            // tx_builder,
            // tx_broadcaster,
            // notification_tx: tx,
        });
        
        // let wallet_handler = WalletNotificationHandler::new(
        //     wallet_interface.clone(),
        //     tx.clone(),
        // );
        
        // blocktalk
        //     .chain()
        //     .register_handler(Arc::new(wallet_handler))
        //     .await;
        
        // blocktalk.chain().subscribe_to_notifications().await?;
        // log::info!("Registered for blockchain notifications");
        
        // Start notification processor
        // let processor = NotificationProcessor::new(wallet_interface.clone(), rx);
        // processor.start();
        
        Ok(wallet_interface)
    }
    
    pub async fn process_block(&self, block: &Block) -> Result<(), WalletError> {
        // let block_height = (self.blocktalk.chain().get_tip().await?).0;
        // let block_hash = block.block_hash();
        
        // log::debug!("Processing block {} at height {}", block_hash, block_height);
        
        // for tx in &block.txdata {
        //     self.process_transaction(tx, Some(block_height)).await?;
        // }
        
        Ok(())
    }
    
    pub async fn process_transaction(
        &self, 
        tx: &Transaction, 
        block_height: Option<i32>
    ) -> Result<(), WalletError> {
        let txid = tx.txid();
        log::debug!("Processing transaction {}", txid);
        
        let wallet = self.wallet.write().unwrap();
        
        // Check if any output script belongs to us
        let is_relevant = tx.output.iter()
            .any(|output| wallet.is_mine(output.script_pubkey.clone()));
            
        if is_relevant {
            log::info!("Found relevant transaction: {}", txid);
            
            // Store transaction metadata
            let timestamp = chrono::Utc::now().timestamp() as u64;
            let metadata = TransactionMetadata {
                timestamp,
                block_height: block_height.map(|h| h as u32),
                fee: None,
                comment: String::new(),
                label: String::new(),
            };
            
            self.db.store_tx_metadata(&txid, &metadata)?;
        }
        
        Ok(())
    }
    
    /// Synchronize the wallet with the blockchain
    pub async fn sync(&self) -> Result<(), WalletError> {
        log::info!("Syncing wallet with blockchain");
        
        // Get current tip from node
        // let (tip_height, _) = self.blocktalk.chain().get_tip().await?;
        // log::debug!("Current blockchain tip is at height {}", tip_height);
        
        // // Rescan blockchain for relevant transactions
        // let start_height = 0; // In a real implementation, we would store the last synced height
        
        // for height in start_height..=tip_height {
        //     // Get block at height
        //     if let Ok((_, tip_hash)) = self.blocktalk.chain().get_tip().await {
        //         if let Ok(block) = self.blocktalk.chain().get_block(&tip_hash, height).await {
        //             // Process the block
        //             self.process_block(&block).await?;
        //         }
        //     }
        // }
        
        log::info!("Wallet sync completed");
        Ok(())
    }
    
    /// Get a new receiving address from the wallet
    pub fn get_new_address(&self, label: Option<&str>) -> Result<Address, WalletError> {
        let mut wallet = self.wallet.write().unwrap();
        let address_info = wallet.reveal_next_address(KeychainKind::External);
        
        if let Some(label_text) = label {
            log::debug!("Labeling address {} as '{}'", address_info.address, label_text);
        }
        
        Ok(address_info.address)
    }

    pub fn get_balance(&self) -> Result<WalletBalance, WalletError> {
        let wallet = self.wallet.read().unwrap();
        let bdk_balance = wallet.balance();
            
        Ok(WalletBalance {
            confirmed: bdk_balance.confirmed,
            unconfirmed: bdk_balance.untrusted_pending,
            immature: bdk_balance.immature,
            total: bdk_balance.confirmed + bdk_balance.untrusted_pending + bdk_balance.immature,
        })
    }
    
    // pub fn create_transaction(
    //     &self,
    //     recipients: &[TxRecipient],
    //     fee_rate: Option<f64>,
    //     subtract_fee_from: Option<Vec<usize>>,
    // ) -> Result<(Transaction, Psbt), WalletError> {
    //     let mut wallet = self.wallet.write().unwrap();
    //     let mut tx_builder = wallet.build_tx();

    //     for (i, recipient) in recipients.iter().enumerate() {
    //         tx_builder.add_recipient(recipient.script.clone(), recipient.amount);
    //         if subtract_fee_from.as_ref().map_or(false, |v| v.contains(&i)) {
    //             tx_builder.subtract_fee_from_last();
    //         }
    //     }

    //     if let Some(rate) = fee_rate {
    //         tx_builder.fee_rate(bdk_wallet::FeeRate::from_sat_per_vb(rate));
    //     }

    //     let (psbt, _) = tx_builder.finish().map_err(|e| WalletError::BDKError(e))?;
    //     let signed_psbt = wallet.sign(psbt.clone()).map_err(|e| WalletError::BDKError(e))?;
    //     let tx = signed_psbt.extract_tx();

    //     Ok((tx, signed_psbt))
    //     Ok((Transaction::default(), Psbt::default()))
    // }

    // pub fn sign_transaction(&self, tx: &mut Transaction) -> Result<bool, WalletError> {
    //     // let mut wallet = self.wallet.write().unwrap();
    //     // Ok(wallet.sign(tx)?)
    //     Ok(true)
    // }

    // pub async fn send_transaction(&self, tx: &Transaction) -> Result<Txid, WalletError> {
    //     let txid = tx.txid();
    //     log::info!("Broadcasting transaction: {}", txid);
    //     // TODO: Implement actual broadcasting
    //     Ok(txid)
    // }
}

fn generate_descriptors(network: Network) -> Result<(String, String), WalletError> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let mut rng = rand::thread_rng();
    let xprv = bitcoin::bip32::ExtendedPrivKey::new_master(network, &mut rng.gen::<[u8; 32]>())
        .map_err(|e| WalletError::Generic(format!("Failed to generate master key: {}", e)))?;

    let external = format!("wpkh({}/0/*)", xprv);
    let internal = format!("wpkh({}/1/*)", xprv);

    Ok((external, internal))
}