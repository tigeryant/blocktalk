use bitcoin::BlockHash;
use blocktalk::{BlockTalk, BlockTalkError};
use std::path::Path;
use std::time::Duration;
use tokio::task::LocalSet;

#[tokio::main]
async fn main() -> Result<(), BlockTalkError> {
    let socket_path = "../bitcoin/datadir_blocktalk/regtest/node.sock";

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

            let chain = blocktalk.chain();

            // Execute chain queries
            let tip_info = query_chain_tip(chain).await;
            
            // If we got the tip info, try to get a block from a few blocks back
            if let Some((height, tip_hash)) = tip_info {
                if height > 3 {
                    // Try to get block from 1 block before tip
                    get_block_at_height(chain, &tip_hash, height).await;
                }
            }

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

/// Queries and displays the current chain tip
async fn query_chain_tip(chain: &blocktalk::ChainInterface) -> Option<(i32, BlockHash)> {
    println!("\n╔════════════════════════════════════════════════════════════════════════════╗");
    println!("║                              Current Chain Tip                             ║");
    println!("╠════════════════════════════════════════════════════════════════════════════╣");
    
    match tokio::time::timeout(Duration::from_secs(5), chain.get_tip()).await {
        Ok(Ok((height, hash))) => {
            println!("║ Height │ {:<65} ║", height);
            println!("╟────────┼───────────────────────────────────────────────────────────────────╢");
            println!("║ Hash   │ {:<65} ║", hash);
            println!("╚════════╧═══════════════════════════════════════════════════════════════════╝");
            Some((height, hash))
        }
        Ok(Err(e)) => {
            println!("║ Error fetching chain tip: {:<49} ║", e);
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
            None
        }
        Err(_) => {
            println!("║ Request timed out after 5 seconds                                         ║");
            println!("╚═════════════════════════════════════════════════════════════════════════════╝");
            None
        }
    }
}

/// Gets and displays block at specific height using the get_block method
async fn get_block_at_height(chain: &blocktalk::ChainInterface, tip_hash: &BlockHash, height: i32) {
    println!("\n╔═════════════════════════════════════════════════════════════════════════════════╗");
    println!("║                                   Block Details                                 ║");
    println!("╠═════════════════════════════════════════════════════════════════════════════════╣");

    match tokio::time::timeout(Duration::from_secs(5), chain.get_block(tip_hash, height)).await {
        Ok(Ok(block)) => {
            println!("║ Hash         │ {:<64} ║", block.block_hash());
            println!("╟──────────────┼──────────────────────────────────────────────────────────────────╢");
            println!("║ Prev Block   │ {:<64} ║", block.header.prev_blockhash);
            println!("║ Merkle Root  │ {:<64} ║", block.header.merkle_root);
            println!("║ Timestamp    │ {:<64} ║", block.header.time);
            println!("║ Nonce        │ {:<64} ║", block.header.nonce);
            println!("║ TX Count     │ {:<64} ║", block.txdata.len());
            
            if !block.txdata.is_empty() {
                println!("╟──────────────┴──────────────────────────────────────────────────────────────────╢");
                println!("║                                 Transactions                                    ║");
                println!("╠═════════════════════════════════════════════════════════════════════════════════╣");
                
                let count = std::cmp::min(3, block.txdata.len());
                for (i, tx) in block.txdata.iter().take(count).enumerate() {
                    println!("║ TX #{:<3}                                                                         ║", i + 1);
                    println!("║ ├─ TXID      │ {:<64} ║", tx.compute_txid());
                    println!("║ ├─ Inputs    │ {:<64} ║", tx.input.len());
                    println!("║ ├─ Outputs   │ {:<64} ║", tx.output.len());
                    
                    if !tx.output.is_empty() {
                        let sample_output = &tx.output[0];
                        println!("║ └─ Sample Out│ {:<64} ║", format!("{} satoshis", sample_output.value));
                    }
                    
                    if tx.is_coinbase() {
                        println!("║     [Coinbase Transaction]                                                      ║");
                    }
                    
                    if i < count - 1 {
                        println!("╟─────────────────────────────────────────────────────────────────────────────────╢");
                    }
                }

                if block.txdata.len() > count {
                    println!("╟─────────────────────────────────────────────────────────────────────────────────╢");
                    println!("║ ... and {} more transaction(s)                                                ║", 
                        block.txdata.len() - count);
                }
            }

            let serialized_size = bitcoin::consensus::serialize(&block).len();
            println!("╟─────────────────────────────────────────────────────────────────────────────────╢");
            println!("║ Block Size   │ {:<64} ║", format!("{} bytes", serialized_size));
            println!("╚═════════════════════════════════════════════════════════════════════════════════╝");
        }
        Ok(Err(e)) => {
            println!("║ Error fetching block: {:<59} ║", e);
            println!("╚═════════════════════════════════════════════════════════════════════════════════╝");
        }
        Err(_) => {
            println!("║ Request timed out after 5 seconds                                                ║");
            println!("╚═════════════════════════════════════════════════════════════════════════════════╝");
        }
    }
}