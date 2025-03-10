use bdk_wallet::chain::local_chain::CheckPoint;
use bdk_wallet::{KeychainKind, Wallet, LocalOutput};
use bdk_wallet::keys::{DescriptorKey, ExtendedKey, GeneratableKey, GeneratedKey, 
    bip39::{ Mnemonic, Language} 
};
use std::collections::HashMap;

use bitcoin::{psbt::Psbt, Address, Block, Network, Transaction};
use rand::{self, Rng};
use std::path::Path;
use std::sync::{Arc, RwLock, Mutex};
use tokio::sync::mpsc;

// use super::database::WalletDatabase;
use super::notification::NotificationProcessor;
use crate::error::WalletError;
use blocktalk::BlockTalk;
// use super::transaction::{TransactionBuilder, TransactionBroadcaster};
use super::types::{CreateWalletOptions, TransactionMetadata, TxRecipient, WalletBalance};

pub struct WalletInterface {
    wallet: Arc<RwLock<Option<Arc<RwLock<Wallet>>>>>,
    // db: WalletDatabase,
    node_socket: String,  // Store connection string instead of BlockTalk instance
    network: Network,
    wallets: Arc<RwLock<HashMap<String, Arc<RwLock<Wallet>>>>>,
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
        log::info!("Initializing wallet interface with network: {:?}", network);

        let wallet_interface = Arc::new(Self {
            wallet: Arc::new(RwLock::new(None)),
            node_socket: node_socket.to_string(),
            network,
            wallets: Arc::new(RwLock::new(HashMap::new())),
        });

        Ok(wallet_interface)
    }

    async fn get_blocktalk(&self) -> Result<BlockTalk, WalletError> {
        BlockTalk::init(&self.node_socket).await.map_err(WalletError::from)
    }

    pub fn create_wallet(&self, options: CreateWalletOptions) -> Result<(), WalletError> {
        let (external_descriptor, internal_descriptor) = if options.blank {
            ("wpkh()".to_string(), "wpkh()".to_string())
        } else {
            generate_descriptors(self.network)?
        };

        let mut wallet_builder = Wallet::create(external_descriptor, internal_descriptor)
            .network(self.network);

        if options.disable_private_keys {
            // wallet_builder = wallet_builder.disable_private_keys();
        }

        if options.descriptors {
            // wallet_builder = wallet_builder.descriptors();
        }

        let wallet = Arc::new(RwLock::new(wallet_builder.create_wallet_no_persist().unwrap()));

        // Store the wallet under the given name and set as current wallet
        let wallet_name = options.wallet_name.clone();
        {
            let mut wallets = self.wallets.write().unwrap();
            wallets.insert(wallet_name.clone(), wallet.clone());
        }
        {
            let mut current_wallet = self.wallet.write().unwrap();
            *current_wallet = Some(wallet);
        }

        log::info!("Created wallet: {}", wallet_name);
        Ok(())
    }

    pub async fn load_wallet(&self, wallet_name: &str) -> Result<(), WalletError> {
        let wallet = {
            let wallets = self.wallets.read().unwrap(); // Use read lock for lookup
            wallets
                .get(wallet_name)
                .cloned() // Clone the Wallet to use outside the lock
                .ok_or_else(|| WalletError::Generic(format!("Wallet not found: {}", wallet_name)))?
        };
    
        {
            let mut current_wallet = self.wallet.write().unwrap();
            *current_wallet = Some(wallet);
        }
    
        log::info!("Loaded wallet: {}", wallet_name);
        self.sync_wallet().await?;
        Ok(())
    }

    pub fn get_current_wallet(&self) -> Result<Arc<RwLock<Wallet>>, WalletError> {
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
        let wallet_guard = wallet.write().unwrap();

        // Check if any output script belongs to us
        let is_relevant = tx
            .output
            .iter()
            .any(|output| wallet_guard.is_mine(output.script_pubkey.clone()));

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

            // self.db.store_tx_metadata(&txid, &metadata)?;
        }

        Ok(())
    }

    pub async fn sync_wallet(&self) -> Result<(), WalletError> {
        log::info!("Syncing wallet with blockchain");

        let blocktalk = self.get_blocktalk().await?;
        let (tip_height, tip_hash) = blocktalk.chain().get_tip().await?;
        log::info!("Current blockchain tip is at height {}", tip_height);

        let wallet = self.get_current_wallet()?;
        let mut wallet_guard = wallet.write().unwrap();
        let wallet_tip: CheckPoint = wallet_guard.latest_checkpoint();
        log::info!(
            "Current wallet tip is: {} at height {}",
            &wallet_tip.hash(),
            &wallet_tip.height()
        );

        let start_height = 0;

        for height in start_height..=tip_height {
            if let Ok(block) = blocktalk.chain().get_block(&tip_hash, height).await {
                log::info!("Applying block at height {}", height);
                wallet_guard.apply_block(&block, height as u32).unwrap();
            }
        }

        //TODO: Sync with mempool
        log::info!("Wallet sync completed");
        Ok(())
    }

    pub fn get_new_address(&self, label: Option<&str>) -> Result<Address, WalletError> {
        let wallet = self.get_current_wallet()?;
        let mut wallet_guard = wallet.write().unwrap();
        let address_info = wallet_guard.reveal_next_address(KeychainKind::External);

        if let Some(label_text) = label {
            log::debug!(
                "Labeling address {} as '{}'",
                address_info.address,
                label_text
            );
        }

        Ok(address_info.address)
    }

    pub fn get_balance(&self) -> Result<WalletBalance, WalletError> {
        let wallet = self.get_current_wallet()?;
        let wallet_guard = wallet.read().unwrap();
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
        let wallet_guard = wallet.read().unwrap();
        Ok(wallet_guard.list_unspent().collect())
    }

    pub fn list_transactions(&self) -> Result<Vec<Transaction>, WalletError> {
        let wallet = self.get_current_wallet()?;
        let wallet_guard = wallet.read().unwrap();
        Ok(wallet_guard.transactions().map(|tx| (*tx.tx_node.tx).clone()).collect())
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
