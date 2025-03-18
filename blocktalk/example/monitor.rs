// examples/monitor.rs
use async_trait::async_trait;
use blocktalk::{BlockTalk, BlockTalkError, ChainNotification, NotificationHandler};
use std::sync::Arc;
use tokio::sync::Mutex;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local = tokio::task::LocalSet::new();

    local
        .run_until(async move {
            println!("⏳ Connecting to Bitcoin node...");
            let blocktalk =
                BlockTalk::init("../bitcoin/datadir_blocktalk/regtest/node.sock").await?;
            println!("✅ Connected successfully!");

            // Get current tip info
            let (height, _) = blocktalk.chain().get_tip().await?;

            // Create monitor
            let monitor = BlockMonitor {
                latest_height: Arc::new(Mutex::new(height)),
            };

            // Register handler with chain interface
            let monitor_arc = Arc::new(monitor);
            blocktalk
                .chain()
                .add_notification_handler(monitor_arc)
                .await
                .map_err(|e| {
                    log::error!("Failed to register notification handler: {}", e);
                    e
                })?;
            // Start subscribing to notifications
            blocktalk.chain().begin_chain_updates().await?;

            println!("🔍 Monitoring blockchain events. Press Ctrl+C to exit.");

            // Keep the program running until Ctrl+C
            tokio::signal::ctrl_c().await?;
            println!("Shutting down monitor...");

            Ok(())
        })
        .await
}
