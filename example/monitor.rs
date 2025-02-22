// examples/monitor.rs
use blocktalk::{BlockTalk, NotificationHandler, ChainNotification, BlockTalkError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

struct BlockMonitor {
    latest_height: Arc<Mutex<i32>>,
}

#[async_trait]
impl NotificationHandler for BlockMonitor {
    async fn handle_notification(&self, notification: ChainNotification) -> Result<(), BlockTalkError> {
        match notification {
            ChainNotification::BlockConnected(block) => {
                let mut height = self.latest_height.lock().await;
                *height += 1;
                println!("New block at height {}: {}", *height, block.block_hash());
            }
            ChainNotification::TransactionAddedToMempool(tx) => {
                println!("New mempool transaction: {}", tx.txid());
            }
            ChainNotification::UpdatedBlockTip(hash) => {
                println!("Block tip updated: {}", hash);
            }
            ChainNotification::BlockDisconnected(hash) => {
                let mut height = self.latest_height.lock().await;
                *height -= 1;
                println!("Block disconnected at height {}: {}", *height, hash);
            }
            ChainNotification::TransactionRemovedFromMempool(txid) => {
                println!("Transaction removed from mempool: {}", txid);
            }
            ChainNotification::ChainStateFlushed => {
                println!("Chain state flushed");
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize BlockTalk
    let mut blocktalk = BlockTalk::init("../bitcoin/datadir_bdk_wallet/regtest/node.sock").await?;

    // Get current tip info
    let (height, _) = blocktalk.chain().get_tip().await?;
    
    // Create monitor
    let monitor = BlockMonitor {
        latest_height: Arc::new(Mutex::new(height)),
    };

    // Register handler with chain interface
    let monitor_arc = Arc::new(monitor);
    blocktalk.chain_mut().register_handler(monitor_arc).await;

    println!("Monitoring blockchain events. Press Ctrl+C to exit.");
    
    // Keep the program running until Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("Shutting down monitor...");

    Ok(())
}