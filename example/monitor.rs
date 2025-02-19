// // examples/monitor.rs
// use blocktalk::{BlockTalk, NotificationHandler, ChainNotification};
// use async_trait::async_trait;
// use std::sync::Arc;
// use tokio::sync::Mutex;

// struct BlockMonitor {
//     latest_height: Arc<Mutex<u32>>,
// }

// #[async_trait]
// impl NotificationHandler for BlockMonitor {
//     async fn handle_notification(&self, notification: ChainNotification) -> Result<(), blocktalk::BlockTalkError> {
//         match notification {
//             ChainNotification::BlockConnected(block) => {
//                 let height = *self.latest_height.lock().await;
//                 println!("New block at height {}: {}", height + 1, block.block_hash());
//                 *self.latest_height.lock().await = height + 1;
//             }
//             ChainNotification::TransactionAddedToMempool(tx) => {
//                 println!("New mempool transaction: {}", tx.txid());
//             }
//             _ => {}
//         }
//         Ok(())
//     }
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let blocktalk = BlockTalk::connect("/path/to/socket").await?;

//     // Get current height
//     let tip = blocktalk.chain().get_tip().await?;
//     let monitor = BlockMonitor {
//         latest_height: Arc::new(Mutex::new(tip.height)),
//     };

//     // Register for notifications
//     blocktalk.register_notifications(monitor).await?;

//     // Keep the program running
//     loop {
//         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//     }
// }
