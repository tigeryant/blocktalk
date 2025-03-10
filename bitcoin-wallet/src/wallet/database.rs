use bdk_wallet::rusqlite;
use bdk_wallet::KeychainKind;
use bdk_wallet::{PersistedWallet, Wallet};
use bitcoin::Network;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::error::WalletError;

const EXTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPdJuLWWArdBsWjqDA3W5WoREnfdgKEcCQB1FMKfSoaFz9JHZU71HwXAqTsjHripkLM62kUQar14SDD8brsmhFKqVUPXGrZLc/86'/1'/0'/0/*)#fv8tutn2";
const INTERNAL_DESCRIPTOR: &str = "tr(tprv8ZgxMBicQKsPdJuLWWArdBsWjqDA3W5WoREnfdgKEcCQB1FMKfSoaFz9JHZU71HwXAqTsjHripkLM62kUQar14SDD8brsmhFKqVUPXGrZLc/86'/1'/0'/1/*)#ccz2p7rj";

// Define ThreadSafeWallet as a Mutex-wrapped PersistedWallet
pub type ThreadSafeWallet = Mutex<PersistedWallet<rusqlite::Connection>>;

pub struct WalletDatabase {
    /// Path to the SQLite database file
    db_path: PathBuf,
}

impl WalletDatabase {
    pub fn new(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    pub fn path(&self) -> &Path {
        &self.db_path
    }

    pub fn open_connection(&self) -> Result<rusqlite::Connection, WalletError> {
        rusqlite::Connection::open(&self.db_path)
            .map_err(|e| WalletError::DatabaseError(format!("Failed to open database: {}", e)))
    }

    pub fn exists(&self) -> bool {
        self.db_path.exists()
    }

    pub fn load_wallet(&self, network: Network) -> Result<ThreadSafeWallet, WalletError> {
        let mut conn = self.open_connection()?;

        let persisted = Wallet::load()
            .descriptor(KeychainKind::External, Some(EXTERNAL_DESCRIPTOR))
            .descriptor(KeychainKind::Internal, Some(INTERNAL_DESCRIPTOR))
            .extract_keys()
            .check_network(network)
            .load_wallet(&mut conn)
            .map_err(|e| WalletError::Generic(format!("Failed to load wallet: {}", e)))?;

        match persisted {
            Some(persisted_wallet) => Ok(Mutex::new(persisted_wallet)),
            None => Err(WalletError::Generic(
                "No wallet found in database".to_string(),
            )),
        }
    }

    pub fn create_wallet(
        &self,
        external_descriptor: String,
        internal_descriptor: String,
        network: Network,
    ) -> Result<ThreadSafeWallet, WalletError> {
        let mut conn = self.open_connection()?;
        let persisted = Wallet::create(EXTERNAL_DESCRIPTOR, INTERNAL_DESCRIPTOR)
            .network(network)
            .create_wallet(&mut conn)
            .map_err(|e| WalletError::Generic(format!("Failed to create wallet: {}", e)))?;

        Ok(Mutex::new(persisted))
    }
}
