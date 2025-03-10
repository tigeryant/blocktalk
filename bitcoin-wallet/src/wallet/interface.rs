use bdk_wallet::{KeychainKind, LocalOutput};
use bitcoin::{Address, Network, Transaction};
use rand::{self, Rng};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Mutex};

use super::database::WalletDatabase;
use super::notification::NotificationProcessor;
use crate::error::WalletError;
use blocktalk::BlockTalk;
// use super::transaction::{TransactionBuilder, TransactionBroadcaster};
use super::types::{CreateWalletOptions, TransactionMetadata, TxRecipient, WalletBalance};
use super::database::ThreadSafeWallet;

pub struct WalletInterface {
    wallet: Arc<RwLock<Option<Arc<ThreadSafeWallet>>>>,
    database: WalletDatabase,
    node_socket: String,
    network: Network,
}

impl WalletInterface {
    pub async fn new(
        wallet_path: &Path,
        node_socket: &str,
        network: Network,
    ) -> Result<Arc<Self>, WalletError> {
        log::info!("Initializing wallet interface with network: {:?}", network);
        
        if let Some(parent) = wallet_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| WalletError::Generic(format!("Failed to create wallet directory: {}", e)))?;
        }
        
        let database = WalletDatabase::new(wallet_path.to_path_buf());

        let wallet_interface = Arc::new(Self {
            wallet: Arc::new(RwLock::new(None)),
            database,
            node_socket: node_socket.to_string(),
            network,
        });

        Ok(wallet_interface)
    }

    pub fn create_wallet(&self, options: CreateWalletOptions) -> Result<(), WalletError> {
        let (external_descriptor, internal_descriptor) = if options.blank {
            ("wpkh()".to_string(), "wpkh()".to_string())
        } else {
            generate_descriptors(self.network)?
        };

        let persisted_wallet = self.database.create_wallet(
            external_descriptor,
            internal_descriptor,
            self.network,
        )?;
        
        let wallet = Arc::new(persisted_wallet); // Wrap in Arc directly
        {
            let mut current_wallet = self.wallet.write().unwrap();
            *current_wallet = Some(wallet);
        }

        log::info!("Created wallet");
        Ok(())
    }

    pub async fn load_wallet(&self, _wallet_name: &str) -> Result<(), WalletError> {
        let persisted_wallet = self.database.load_wallet(self.network)?;
        let wallet = Arc::new(persisted_wallet); // Wrap in Arc directly
        
        {
            let mut current_wallet = self.wallet.write().unwrap();
            *current_wallet = Some(wallet);
        }
        
        log::info!("Loaded wallet from database");
        self.sync_wallet().await
    }

    async fn get_blocktalk(&self) -> Result<BlockTalk, WalletError> {
        BlockTalk::init(&self.node_socket).await.map_err(WalletError::from)
    }

    fn get_current_wallet(&self) -> Result<Arc<ThreadSafeWallet>, WalletError> {
        let wallet_lock = self.wallet.read().unwrap();
        wallet_lock.clone().ok_or(WalletError::Generic("No wallet loaded".to_string()))
    }

    pub async fn process_transaction(
        &self,
        tx: &Transaction,
        block_height: Option<i32>,
    ) -> Result<(), WalletError> {
        let txid = tx.txid();
        log::debug!("Processing transaction {}", txid);

        let wallet = self.get_current_wallet()?;
        let wallet_guard = wallet.lock().unwrap();

        // Check if any output script belongs to us
        let is_relevant = tx
            .output
            .iter()
            .any(|output| wallet_guard.is_mine(output.script_pubkey.clone()));

        if is_relevant {
            log::info!("Found relevant transaction: {}", txid);
            
            // Apply transaction to wallet
            if let Some(height) = block_height {
                // Transaction is confirmed
                // In a real implementation, you would need the full block to apply
                log::info!("Transaction is confirmed at height {}", height);
            } else {
                // Transaction is unconfirmed
                log::info!("Transaction is unconfirmed");
            }
            
            // Persist changes to database
            // wallet_guard.persist(wallet_guard.connection())?;

            // Store transaction metadata
            let timestamp = chrono::Utc::now().timestamp() as u64;
            let metadata = TransactionMetadata {
                timestamp,
                block_height: block_height.map(|h| h as u32),
                fee: None,
                comment: String::new(),
                label: String::new(),
            };

            // TODO: Store metadata somewhere
            // self.db.store_tx_metadata(&txid, &metadata)?;
        }

        Ok(())
    }

    pub async fn sync_wallet(&self) -> Result<(), WalletError> {
        log::info!("Syncing wallet with blockchain");

        let blocktalk = self.get_blocktalk().await?;
        let (tip_height, tip_hash) = blocktalk.chain().get_tip().await?;
        log::info!("Current blockchain tip is at height {} with hash {}", tip_height, tip_hash);

        let wallet = self.get_current_wallet()?;
        let mut wallet_guard = wallet.lock().unwrap();
        let wallet_tip = wallet_guard.latest_checkpoint();
        log::info!(
            "Wallet tip is: {} at height {}",
            &wallet_tip.hash(),
            &wallet_tip.height()
        );

        let start_height = wallet_tip.height() as i32 + 1;

        log::info!("🔄 Syncing wallet with blockchain");
        for height in start_height..=tip_height {
            if let Ok(block) = blocktalk.chain().get_block(&tip_hash, height).await {
                wallet_guard.apply_block(&block, height as u32)
                    .map_err(|e| WalletError::Generic(format!("Failed to apply block: {}", e)))?;
            }
        }

        log::info!("✅ Wallet sync completed");
        let wallet_tip = wallet_guard.latest_checkpoint();
        log::info!(
            "Wallet tip is: {} at height {}",
            &wallet_tip.hash(),
            &wallet_tip.height()
        );
        Ok(())
    }

    pub fn get_new_address(&self, label: Option<&str>) -> Result<Address, WalletError> {
        let wallet = self.get_current_wallet()?;
        let mut wallet_guard = wallet.lock().unwrap();
        let address_info = wallet_guard.reveal_next_address(KeychainKind::External);
        
        // Persist changes to database
        // wallet_guard.persist(wallet_guard.connection())?;

        if let Some(label_text) = label {
            log::debug!(
                "Labeling address {} as '{}'",
                address_info.address,
                label_text
            );
            // TODO: Store label somewhere
        }

        Ok(address_info.address)
    }

    pub fn get_balance(&self) -> Result<WalletBalance, WalletError> {
        let wallet = self.get_current_wallet()?;
        let wallet_guard = wallet.lock().unwrap();
        let bdk_balance = wallet_guard.balance();

        Ok(WalletBalance {
            confirmed: bdk_balance.confirmed,
            unconfirmed: bdk_balance.untrusted_pending,
            immature: bdk_balance.immature,
            total: bdk_balance.confirmed + bdk_balance.untrusted_pending + bdk_balance.immature,
        })
    }

    pub fn list_unspent(&self) -> Result<Vec<LocalOutput>, WalletError> {
        let wallet = self.get_current_wallet()?;
        let wallet_guard = wallet.lock().unwrap();
        Ok(wallet_guard.list_unspent().collect())
    }

    pub fn list_transactions(&self) -> Result<Vec<Transaction>, WalletError> {
        let wallet = self.get_current_wallet()?;
        let wallet_guard = wallet.lock().unwrap();
        Ok(wallet_guard.transactions().map(|tx| (*tx.tx_node.tx).clone()).collect())
    }

    pub async fn rescan_blockchain(&self, start_height: i32, stop_height: Option<i32>) -> Result<(i32, i32), WalletError> {
        log::info!("Rescanning blockchain from height {} to {:?}", start_height, stop_height);
        
        let blocktalk = self.get_blocktalk().await?;
        let (tip_height, tip_hash) = blocktalk.chain().get_tip().await?;
        log::info!("Current blockchain tip is at height {}", tip_height);
        
        // Determine actual stop height (default to chain tip if not specified)
        let actual_stop_height = stop_height.unwrap_or(tip_height);
        // Cap at chain tip
        let actual_stop_height = std::cmp::min(actual_stop_height, tip_height);
        
        let wallet = self.get_current_wallet()?;
        let mut wallet_guard = wallet.lock().unwrap();
        
        // For a full rescan from a specific height, we might need to disconnect blocks 
        // and reset the wallet state to that height first
        if start_height == 0 {
            // Full rescan from genesis
            log::info!("Performing full rescan from genesis");
            // Reset wallet state would happen here in a complete implementation
            // wallet_guard.reset_to_height(0)?;
        } else if start_height > 0 {
            // Partial rescan from a specific height
            log::info!("Performing partial rescan from height {}", start_height);
            // In a real implementation, we might need to disconnect blocks after this height
            // wallet_guard.reset_to_height(start_height as u32)?;
        }
        
        // Process blocks in the specified range
        for height in start_height..=actual_stop_height {
            if let Ok(block) = blocktalk.chain().get_block(&tip_hash, height as i32).await {
                wallet_guard.apply_block(&block, height as u32)
                    .map_err(|e| WalletError::Generic(format!("Failed to apply block during rescan: {}", e)))?;
            } else {
                log::warn!("Failed to retrieve block at height {}", height);
            }
        }
        
        log::info!("Blockchain rescan completed from {} to {}", start_height, actual_stop_height);
        Ok((start_height, actual_stop_height))
    }
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