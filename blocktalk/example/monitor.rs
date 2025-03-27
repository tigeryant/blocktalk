// examples/monitor.rs
use async_trait::async_trait;
use blocktalk::{BlockTalk, BlockTalkError, ChainNotification, NotificationHandler};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::Path;
use std::time::Duration;
use tokio::task::LocalSet;

struct BlockMonitor {
    latest_height: Arc<Mutex<i32>>,
}

#[async_trait]
impl NotificationHandler for BlockMonitor {
    async fn handle_notification(
        &self,
        notification: ChainNotification,
    ) -> Result<(), BlockTalkError> {
        match notification {
            ChainNotification::UpdatedBlockTip(_) => {
                println!("\n╔═══════════════════════╗");
                println!("║   Block Tip Updated   ║");
                println!("╚═══════════════════════╝");
            }

            ChainNotification::BlockConnected(block) => {
                let mut height = self.latest_height.lock().await;
                *height += 1;

                println!("\n╔════════════════════════════════════════════════════════════════════════════════╗");
                println!("║                                New Block Connected                             ║");
                println!("╠════════════════════════════════════════════════════════════════════════════════╣");
                println!("║ Height      │ {:<64} ║", *height);
                println!("║ Hash        │ {:<64} ║", block.block_hash());
                println!("║ Time        │ {:<64} ║", block.header.time);
                println!("║ Transaction │ {:<64} ║", block.txdata.len());
                println!("║ Size        │ {:<64} ║", format!("{} bytes", bitcoin::consensus::serialize(&block).len()));
                println!("╚═════════════╧══════════════════════════════════════════════════════════════════╝");
            }
            
            ChainNotification::TransactionAddedToMempool(tx) => {
                println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
                println!("║                         Transaction Added to Mempool                         ║");
                println!("╠══════════════════════════════════════════════════════════════════════════════╣");
                println!("║ TXID         │ {:<60} ║", tx.compute_txid());
                println!("║ Inputs       │ {:<60} ║", tx.input.len());
                println!("║ Outputs      │ {:<60} ║", tx.output.len());
                if tx.is_coinbase() {
                    println!("║ Type         │ {:<60} ║", "Coinbase Transaction");
                }
                println!("╚══════════════╧═══════════════════════════════════════════════════════════════╝");
            }
            
            ChainNotification::BlockDisconnected(hash) => {
                let mut height = self.latest_height.lock().await;
                *height -= 1;
                println!("\n╔════════════════════════════════════════════════════════════════════════╗");
                println!("║                          Block Disconnected                            ║");
                println!("╠════════════════════════════════════════════════════════════════════════╣");
                println!("║ Height       │ {:<60} ║", *height);
                println!("║ Hash         │ {:<60} ║", hash);
                println!("╚══════════════╧══════════════════════════════════════════════════════════╝");
            }
            
            ChainNotification::TransactionRemovedFromMempool(txid) => {
                println!("\n╔════════════════════════════════════════════════════════════════════════╗");
                println!("║                    Transaction Removed from Mempool                    ║");
                println!("╠════════════════════════════════════════════════════════════════════════╣");
                println!("║ TXID         │ {:<60} ║", txid);
                println!("╚══════════════╧══════════════════════════════════════════════════════════╝");
            }
            
            ChainNotification::ChainStateFlushed => {
                println!("\n╔════════════════════════════════════════════╗");
                println!("║            Chain State Flushed             ║");
                println!("╚════════════════════════════════════════════╝");
            }
        }
        Ok(())
    }
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

#[tokio::main]
async fn main() -> Result<(), BlockTalkError> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: monitor <socket_path>");
        return Ok(());
    }

    let socket_path = &args[1];

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

            // Create and register notification handler
            let handler = Arc::new(BlockMonitor {
                latest_height: Arc::new(Mutex::new(0)),
            });
            chain.add_notification_handler(handler.clone()).await?;

            // Start receiving chain updates
            chain.begin_chain_updates().await?;

            println!("Monitoring chain updates. Press Ctrl+C to stop.");
            
            // Keep the program running
            tokio::signal::ctrl_c().await?;
            println!("\nStopping chain updates...");
            
            chain.stop_chain_updates().await?;
            Ok(())
        })
        .await
}
