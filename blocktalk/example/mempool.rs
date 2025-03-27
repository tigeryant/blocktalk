use blocktalk::{BlockTalk, BlockTalkError, MempoolInterface, TransactionAncestry};
use bitcoin::{Transaction, Txid};
use std::path::Path;
use std::time::Duration;
use std::str::FromStr;
use tokio::task::LocalSet;

#[tokio::main]
async fn main() -> Result<(), BlockTalkError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <socket_path> <transaction_id>", args[0]);
        println!("Example: {} ../bitcoin/datadir_blocktalk/regtest/node.sock 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", args[0]);
        return Ok(());
    }

    let socket_path = &args[1];
    let txid = Txid::from_str(&args[2])
        .expect("Invalid transaction ID. Must be a 64-character hex string.");

    if !check_socket_path(socket_path) {
        return Ok(());
    }

    let local = LocalSet::new();
    local
        .run_until(async {
            let blocktalk = match connect_to_node(socket_path).await {
                Some(bt) => bt,
                None => return Ok(()),
            };

            let mempool = blocktalk.mempool();

            // Check if transaction is in mempool
            check_transaction_in_mempool(mempool.as_ref(), &txid).await;

            // Check for descendants
            check_transaction_descendants(mempool.as_ref(), &txid).await;

            // Get transaction ancestry
            get_transaction_ancestry(mempool.as_ref(), &txid).await;

            Ok(())
        })
        .await
}

/// Checks if the socket path exists and prints helpful error if not
fn check_socket_path(socket_path: &str) -> bool {
    if Path::new(socket_path).exists() {
        return true;
    }

    println!("Error: Socket file {} does not exist!", socket_path);
    println!("Please check that:");
    println!("1. Bitcoin node is running");
    println!("2. Bitcoin node is configured to use this Unix socket path");
    println!("3. You have the correct permissions to access the socket");
    false
}

/// Attempts to connect to the Bitcoin node with timeout
async fn connect_to_node(socket_path: &str) -> Option<BlockTalk> {
    println!("⏳ Connecting to Bitcoin node...");
    match tokio::time::timeout(Duration::from_secs(5), BlockTalk::init(socket_path)).await {
        Ok(Ok(bt)) => {
            println!("✅ Connected successfully!");
            Some(bt)
        }
        Ok(Err(e)) => {
            println!("⛔️ Error connecting to Bitcoin node: {}", e);
            None
        }
        Err(_) => {
            println!("⏲️ Connection timed out after 5 seconds");
            None
        }
    }
}

/// Checks if a transaction is in the mempool
async fn check_transaction_in_mempool(mempool: &dyn MempoolInterface, txid: &Txid) {
    println!("\n╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              Mempool Status                                ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    
    match tokio::time::timeout(Duration::from_secs(5), mempool.is_in_mempool(txid)).await {
        Ok(Ok(is_in)) => {
            println!("║ Transaction │ {:<65} ║", txid);
            println!("╟────────────┼───────────────────────────────────────────────────────────────────╢");
            println!("║ Status     │ {:<65} ║", if is_in { "In Mempool" } else { "Not in Mempool" });
            println!("╚════════════╧═══════════════════════════════════════════════════════════════╝");
        }
        Ok(Err(e)) => {
            println!("║ Error checking mempool status: {:<45} ║", e);
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
        }
        Err(_) => {
            println!("║ Request timed out after 5 seconds                                         ║");
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
        }
    }
}

/// Checks if a transaction has descendants in the mempool
async fn check_transaction_descendants(mempool: &dyn MempoolInterface, txid: &Txid) {
    println!("\n╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            Transaction Descendants                         ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    
    match tokio::time::timeout(Duration::from_secs(5), mempool.has_descendants_in_mempool(txid)).await {
        Ok(Ok(has_descendants)) => {
            println!("║ Transaction │ {:<65} ║", txid);
            println!("╟────────────┼───────────────────────────────────────────────────────────────────╢");
            println!("║ Status     │ {:<65} ║", if has_descendants { "Has Descendants" } else { "No Descendants" });
            println!("╚════════════╧═══════════════════════════════════════════════════════════════╝");
        }
        Ok(Err(e)) => {
            println!("║ Error checking descendants: {:<51} ║", e);
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
        }
        Err(_) => {
            println!("║ Request timed out after 5 seconds                                         ║");
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
        }
    }
}

/// Gets and displays transaction ancestry information
async fn get_transaction_ancestry(mempool: &dyn MempoolInterface, txid: &Txid) {
    println!("\n╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                            Transaction Ancestry                            ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    
    match tokio::time::timeout(Duration::from_secs(5), mempool.get_transaction_ancestry(txid)).await {
        Ok(Ok(ancestry)) => {
            println!("║ Transaction │ {:<65} ║", txid);
            println!("╟────────────┼───────────────────────────────────────────────────────────────────╢");
            println!("║ Ancestors  │ {:<65} ║", ancestry.ancestors);
            println!("║ Descendants│ {:<65} ║", ancestry.descendants);
            println!("║ Size       │ {:<65} ║", format!("{} bytes", ancestry.ancestor_size));
            println!("║ Fees       │ {:<65} ║", format!("{} satoshis", ancestry.ancestor_fees));
            println!("╚════════════╧═══════════════════════════════════════════════════════════════╝");
        }
        Ok(Err(e)) => {
            println!("║ Error getting ancestry: {:<55} ║", e);
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
        }
        Err(_) => {
            println!("║ Request timed out after 5 seconds                                         ║");
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
        }
    }
} 