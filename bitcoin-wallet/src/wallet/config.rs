use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub keypool_size: u32,
    pub rescan: bool,
    pub timestamp: Option<u64>,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    // "sqlite"
    pub db_type: String,
    pub path: PathBuf,
}
