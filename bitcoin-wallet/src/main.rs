use clap::Command;
use std::path::PathBuf;
use std::process;
use tokio::task::LocalSet;
use env_logger;
use log;
use std::net::SocketAddr;

use bitcoin_wallet::{
    config::Config,
    rpc::RPCServer,
    wallet::WalletInterface,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    log::info!("Starting bitcoin-wallet");

    let matches = Command::new("Bitcoin Wallet")
        .version("0.1.0")
        .about("Bitcoin Core compatible wallet using BDK and blocktalk")
        .arg(
            clap::Arg::new("conf")
                .long("conf")
                .value_name("FILE")
                .help("Specify configuration file (default: bitcoin.conf)")
                .value_parser(clap::value_parser!(String))
        )
        .arg(
            clap::Arg::new("datadir")
                .long("datadir")
                .value_name("DIR")
                .help("Specify data directory")
                .value_parser(clap::value_parser!(String))
        )
        .arg(
            clap::Arg::new("rpcbind")
                .long("rpcbind")
                .value_name("ADDR")
                .help("Bind to given address to listen for JSON-RPC connections")
                .default_value("127.0.0.1")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("rpcport")
                .long("rpcport")
                .value_name("PORT")
                .help("Listen for JSON-RPC connections on PORT")
                .default_value("8332")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("rpcuser")
                .long("rpcuser")
                .value_name("USER")
                .help("Username for JSON-RPC connections")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("rpcpassword")
                .long("rpcpassword")
                .value_name("PASSWORD")
                .help("Password for JSON-RPC connections")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("rpcauth")
                .long("rpcauth")
                .value_name("USER:SALT$HASH")
                .help("Username and HMAC-SHA-256 hashed password for JSON-RPC connections")
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("regtest")
                .long("regtest")
                .help("Use regression test network")
                .value_parser(clap::value_parser!(bool))
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("testnet")
                .long("testnet")
                .help("Use testnet")
                .value_parser(clap::value_parser!(bool))
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            clap::Arg::new("node-socket")
                .long("node-socket")
                .value_name("PATH")
                .help("Path to the Bitcoin node socket")
                .value_parser(clap::value_parser!(String))
                .required(true),
        )
        .arg(
            clap::Arg::new("wallet")
                .long("wallet")
                .value_name("WALLET")
                .help("Specify wallet file (within wallets directory)")
                .value_parser(clap::value_parser!(String)),
        )
        .get_matches();

    let network = if matches.contains_id("regtest") {
        bitcoin::Network::Regtest
    } else if matches.contains_id("testnet") {
        bitcoin::Network::Testnet
    } else {
        bitcoin::Network::Bitcoin
    };

    let data_dir = if let Some(dir) = matches.get_one::<String>("datadir") {
        PathBuf::from(dir)
    } else {
        let home = dirs::home_dir().expect("Failed to determine home directory");
        match network {
            bitcoin::Network::Bitcoin => home.join(".bitcoin"),
            bitcoin::Network::Testnet => home.join(".bitcoin").join("testnet3"),
            bitcoin::Network::Regtest => home.join(".bitcoin").join("regtest"),
            _ => home.join(".bitcoin"),
        }
    };

    let wallet_dir = data_dir.join("wallets");
    let wallet_name = matches.get_one::<String>("wallet").map(String::as_str).unwrap_or("wallet.dat");

    let conf_path = matches
        .get_one::<String>("conf")
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| data_dir.join("bitcoin.conf"));
    
    let config = match Config::load(&conf_path, matches.clone()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Failed to load configuration: {}", err);
            process::exit(1);
        }
    };

    let rpc_addr = format!(
        "{}:{}",
        matches.get_one::<String>("rpcbind").map(String::as_str).unwrap_or(&config.rpc.bind),
        matches.get_one::<String>("rpcport").map(String::as_str).unwrap_or(&config.rpc.port)
    ).parse::<SocketAddr>()
    .unwrap_or_else(|e| {
        eprintln!("Invalid RPC address: {}", e);
        process::exit(1);
    });

    let node_socket = matches.get_one::<String>("node-socket").unwrap().to_string();

    let local = LocalSet::new();
    
    local.run_until(async move {
        std::fs::create_dir_all(&wallet_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create wallet directory: {}", e);
            process::exit(1);
        });

        log::info!("Initializing wallet with network: {:?}", network);
        let wallet_path = wallet_dir.join(wallet_name);
        log::info!("Using wallet at: {}", wallet_path.display());

        let wallet = match WalletInterface::new(&wallet_path, &node_socket, network).await {
            Ok(wallet) => wallet,
            Err(e) => {
                eprintln!("Failed to initialize wallet: {}", e);
                process::exit(1);
            }
        };

        log::info!("Starting RPC server on {}", rpc_addr);
        let mut rpc_server = RPCServer::new(wallet, &config.rpc);
        if let Err(e) = rpc_server.start(rpc_addr).await {
            eprintln!("Failed to start RPC server: {}", e);
            process::exit(1);
        }

        log::info!("Wallet is running. Press Ctrl+C to exit");
        tokio::signal::ctrl_c().await.unwrap();
        
        log::info!("Shutting down wallet");
        rpc_server.stop();
        
        Ok(())
    }).await
}