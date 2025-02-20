use blocktalk::{BlockTalk, BlockTalkError};
use std::path::Path;
use std::time::Duration;
use tokio::task::LocalSet;
use bitcoin::{BlockHash, Block};

#[tokio::main]
async fn main() -> Result<(), BlockTalkError> {
    let socket_path = "../bitcoin/datadir_bdk_wallet/regtest/node.sock";

    if !check_socket_path(socket_path) {
        return Ok(());
    }

    let local = LocalSet::new();
    local.run_until(async {
        let blocktalk = match connect_to_node(socket_path).await {
            Some(bt) => bt,
            None => return Ok(()),
        };

        let chain = blocktalk.chain();
        
        // Execute chain queries
        query_chain_tip(chain).await;
        let genesis_hash = query_genesis_block(chain).await;
        
        if let Some(genesis) = genesis_hash {
            // Retrieve the full genesis block
            get_block_details(chain, &genesis).await;
            
            // Only run this if we got the tip and genesis successfully
            if let Ok((_, tip_hash)) = chain.get_tip().await {
                find_common_ancestor(chain, &tip_hash, &genesis).await;
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
    println!("Connecting to Bitcoin node...");
    match tokio::time::timeout(
        Duration::from_secs(5),
        BlockTalk::init(socket_path),
    ).await {
        Ok(Ok(bt)) => {
            println!("Connected successfully!");
            Some(bt)
        }
        Ok(Err(e)) => {
            println!("Error connecting to Bitcoin node: {}", e);
            None
        }
        Err(_) => {
            println!("Connection timed out after 5 seconds");
            None
        }
    }
}

/// Queries and displays the current chain tip
async fn query_chain_tip(chain: &blocktalk::ChainInterface) {
    println!("\nFetching current chain tip...");
    match tokio::time::timeout(Duration::from_secs(5), chain.get_tip()).await {
        Ok(Ok((height, hash))) => {
            println!("Chain tip:");
            println!("  Height: {}", height);
            println!("  Hash: {}", hash);
        }
        Ok(Err(e)) => {
            println!("Error fetching chain tip: {}", e);
        }
        Err(_) => {
            println!("Request timed out after 5 seconds");
        }
    }
}

/// Queries and displays the genesis block
async fn query_genesis_block(chain: &blocktalk::ChainInterface) -> Option<BlockHash> {
    println!("\nFetching genesis block...");
    match tokio::time::timeout(
        Duration::from_secs(5),
        chain.get_block_at_height(0),
    ).await {
        Ok(Ok(Some(hash))) => {
            println!("Genesis block hash: {}", hash);
            
            // Check if genesis block is in best chain
            match chain.is_in_best_chain(&hash).await {
                Ok(is_in_chain) => {
                    println!("Genesis block is {} the best chain",
                        if is_in_chain { "in" } else { "not in" });
                }
                Err(e) => {
                    println!("Error checking if genesis is in best chain: {}", e);
                }
            }
            
            Some(hash)
        }
        Ok(Ok(None)) => {
            println!("No block found at height 0!");
            None
        }
        Ok(Err(e)) => {
            println!("Error fetching genesis block: {}", e);
            None
        }
        Err(_) => {
            println!("Request timed out after 5 seconds");
            None
        }
    }
}

/// Gets and displays full block details using the get_block_by_hash method
async fn get_block_details(chain: &blocktalk::ChainInterface, block_hash: &BlockHash) {
    println!("\nFetching full block details for {}...", block_hash);
    match tokio::time::timeout(
        Duration::from_secs(5),
        chain.get_block_by_hash(block_hash),
    ).await {
        Ok(Ok(Some(block))) => {
            println!("Block details:");
            println!("  Version: {:?}", block.header.version);
            println!("  Previous block hash: {}", block.header.prev_blockhash);
            println!("  Merkle root: {}", block.header.merkle_root);
            println!("  Timestamp: {}", 
                block.header.time,
            );
            println!("  Bits: 0x{:x}", block.header.bits);
            println!("  Nonce: {}", block.header.nonce);
            println!("  Transaction count: {}", block.txdata.len());
            
            // Display first few transactions
            if !block.txdata.is_empty() {
                let count = std::cmp::min(3, block.txdata.len());
                println!("  First {} transaction(s):", count);
                for (i, tx) in block.txdata.iter().take(count).enumerate() {
                    println!("    {}. TXID: {}", i+1, tx.txid());
                    println!("       Input count: {}", tx.input.len());
                    println!("       Output count: {}", tx.output.len());
                    
                    // Show a sample of transaction outputs if present
                    if !tx.output.is_empty() {
                        let sample_output = &tx.output[0];
                        println!("       Sample output: {} satoshis", sample_output.value);
                        if tx.is_coinbase() {
                            println!("       This is a coinbase transaction");
                        }
                    }
                }
                
                if block.txdata.len() > count {
                    println!("    ... and {} more transaction(s)", block.txdata.len() - count);
                }
            }
            
            // Calculate block size
            let serialized_size = bitcoin::consensus::serialize(&block).len();
            println!("  Block size: {} bytes", serialized_size);
            
            // Show block weight if it's a segwit block
            // if block.weight() != serialized_size * 4 {
            //     println!("  Block weight: {} weight units (segwit)", block.weight());
            // }
        }
        Ok(Ok(None)) => {
            println!("Block not found!");
        }
        Ok(Err(e)) => {
            println!("Error fetching block: {}", e);
        }
        Err(_) => {
            println!("Request timed out after 5 seconds");
        }
    }
}

/// Finds and displays the common ancestor between two blocks
async fn find_common_ancestor(
    chain: &blocktalk::ChainInterface, 
    block1: &BlockHash, 
    block2: &BlockHash
) {
    println!("\nFinding common ancestor between tip and genesis...");
    match tokio::time::timeout(
        Duration::from_secs(5),
        chain.find_common_ancestor(block1, block2),
    ).await {
        Ok(Ok(Some(ancestor))) => {
            println!("Common ancestor: {}", ancestor);
            println!("(Should be genesis block)");
        }
        Ok(Ok(None)) => {
            println!("No common ancestor found!");
        }
        Ok(Err(e)) => {
            println!("Error finding common ancestor: {}", e);
        }
        Err(_) => {
            println!("Request timed out after 5 seconds");
        }
    }
}