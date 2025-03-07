use clap::ArgMatches;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

use crate::error::WalletError;
use crate::rpc::{RpcConfig, RpcAuth};

#[derive(Debug, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    pub rpc: RpcConfig,
    pub wallet: WalletConfig,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub network: bitcoin::Network,
}

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

impl Config {
    /// Load configuration from a file and command line arguments, Bitcoin Core style
    pub fn load(conf_path: &Path, matches: ArgMatches) -> Result<Self, WalletError> {
        // Default configuration
        let mut config = Config {
            network: NetworkConfig {
                network: bitcoin::Network::Bitcoin,
            },
            rpc: RpcConfig {
                bind: "127.0.0.1".to_string(),
                port: "8332".to_string(),
                auth: RpcAuth {
                    user: None,
                    password: None,
                    auth_pairs: Vec::new(),
                },
                allow_ips: vec!["127.0.0.1".to_string()],
            },
            wallet: WalletConfig {
                keypool_size: 1000,
                rescan: false,
                timestamp: None,
                database: DatabaseConfig {
                    db_type: "sqlite".to_string(),
                    path: PathBuf::from("wallet.db"),
                },
            },
        };
        
        // Read configuration file if it exists
        if conf_path.exists() {
            let file = fs::File::open(conf_path)
                .map_err(|e| WalletError::ConfigError(format!("Failed to open config file: {}", e)))?;
            
            let reader = io::BufReader::new(file);
            let mut section = String::new();
            
            for line in reader.lines() {
                let line = line.map_err(|e| WalletError::ConfigError(format!("Failed to read line: {}", e)))?;
                let trimmed = line.trim();
                
                // Skip comments and empty lines
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                
                // Handle section headers
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    section = trimmed[1..trimmed.len() - 1].to_string();
                    continue;
                }
                
                // Process key-value pairs
                if let Some(pos) = trimmed.find('=') {
                    let key = trimmed[..pos].trim();
                    let value = trimmed[pos + 1..].trim();
                    
                    Self::apply_setting(&mut config, &section, key, value)?;
                }
            }
        }
        
        // Override with command line arguments
        Self::apply_command_line_args(&mut config, &matches)?;
        
        Ok(config)
    }
    
    fn apply_setting(config: &mut Config, section: &str, key: &str, value: &str) -> Result<(), WalletError> {
        match (section, key) {
            // Network settings
            ("", "testnet") | ("test", "testnet") => {
                if value == "1" || value.to_lowercase() == "true" {
                    config.network.network = bitcoin::Network::Testnet;
                }
            },
            ("", "regtest") | ("regtest", "") => {
                if value == "1" || value.to_lowercase() == "true" {
                    config.network.network = bitcoin::Network::Regtest;
                }
            },
            
            // RPC settings
            ("", "rpcbind") | ("rpc", "bind") => {
                config.rpc.bind = value.to_string();
            },
            ("", "rpcport") | ("rpc", "port") => {
                config.rpc.port = value.to_string();
            },
            ("", "rpcuser") | ("rpc", "user") => {
                config.rpc.auth.user = Some(value.to_string());
            },
            ("", "rpcpassword") | ("rpc", "password") => {
                config.rpc.auth.password = Some(value.to_string());
            },
            ("", "rpcauth") | ("rpc", "auth") => {
                config.rpc.auth.auth_pairs.push(value.to_string());
            },
            ("", "rpcallowip") | ("rpc", "allowip") => {
                config.rpc.allow_ips.push(value.to_string());
            },
            
            // Wallet settings
            ("wallet", "keypool") => {
                if let Ok(size) = value.parse::<u32>() {
                    config.wallet.keypool_size = size;
                }
            },
            ("wallet", "rescan") => {
                if value == "1" || value.to_lowercase() == "true" {
                    config.wallet.rescan = true;
                }
            },
            ("wallet", "timestamp") => {
                if let Ok(ts) = value.parse::<u64>() {
                    config.wallet.timestamp = Some(ts);
                }
            },
            ("wallet", "dbtype") => {
                config.wallet.database.db_type = value.to_string();
            },
            
            // Ignore unknown settings
            _ => {
                log::debug!("Ignoring unknown config option: [{}] {}", section, key);
            }
        }
        
        Ok(())
    }
    
    fn apply_command_line_args(config: &mut Config, matches: &ArgMatches) -> Result<(), WalletError> {
        // Network settings
        if matches.contains_id("testnet") {
            config.network.network = bitcoin::Network::Testnet;
        }
        if matches.contains_id("regtest") {
            config.network.network = bitcoin::Network::Regtest;
        }
        
        // RPC settings
        if let Some(bind) = matches.get_one::<String>("rpcbind") {
            config.rpc.bind = bind.clone();
        }
        if let Some(port) = matches.get_one::<String>("rpcport") {
            config.rpc.port = port.clone();
        }
        if let Some(user) = matches.get_one::<String>("rpcuser") {
            config.rpc.auth.user = Some(user.clone());
        }
        if let Some(password) = matches.get_one::<String>("rpcpassword") {
            config.rpc.auth.password = Some(password.clone());
        }
        if let Some(auth) = matches.get_one::<String>("rpcauth") {
            config.rpc.auth.auth_pairs.push(auth.clone());
        }
        
        Ok(())
    }
}