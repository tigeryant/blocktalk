//! Blockchain notification processing for wallet updates

use async_trait::async_trait;
use blocktalk::{ChainNotification, NotificationHandler};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use super::interface::WalletInterface;
use super::types::WalletEvent;
use crate::error::WalletError;

/// Handler for blockchain notifications
#[derive(Clone)]
pub(crate) struct WalletNotificationHandler {
    wallet: Arc<WalletInterface>,
    tx: mpsc::Sender<WalletEvent>,
}

impl WalletNotificationHandler {
    pub fn new(wallet: Arc<WalletInterface>, tx: mpsc::Sender<WalletEvent>) -> Self {
        Self { wallet, tx }
    }
}

// #[async_trait]
// impl NotificationHandler for WalletNotificationHandler {
//     async fn handle_notification(
//         &self,
//         notification: ChainNotification,
//     ) -> Result<(), blocktalk::BlockTalkError> {
//         match notification {
//             ChainNotification::BlockConnected(block) => {
//                 if let Err(e) = self.tx.send(WalletEvent::BlockConnected(block.clone())).await {
//                     log::error!("Failed to send block connected event: {}", e);
//                 }
//             }
//             ChainNotification::BlockDisconnected(hash) => {
//                 if let Err(e) = self.tx.send(WalletEvent::BlockDisconnected(hash)).await {
//                     log::error!("Failed to send block disconnected event: {}", e);
//                 }
//             }
//             ChainNotification::TransactionAddedToMempool(tx) => {
//                 if let Err(e) = self.tx.send(WalletEvent::TransactionDetected(tx)).await {
//                     log::error!("Failed to send transaction detected event: {}", e);
//                 }
//             }
//             _ => {}
//         }
//         Ok(())
//     }
// }

pub struct NotificationProcessor {
    wallet: Arc<WalletInterface>,
    rx: Arc<Mutex<mpsc::Receiver<WalletEvent>>>,
}

// impl NotificationProcessor {
//     /// Create a new notification processor
//     pub fn new(wallet: Arc<WalletInterface>, rx: mpsc::Receiver<WalletEvent>) -> Self {
//         Self {
//             wallet,
//             rx: Arc::new(Mutex::new(rx)),
//         }
//     }

//     /// Start processing notifications in a background task
//     pub fn start(&self) {
//         let wallet = self.wallet.clone();
//         let rx = self.rx.clone();

//         tokio::task::spawn(async move {
//             let mut rx_guard = rx.lock().unwrap().clone();

//             while let Some(event) = rx_guard.recv().await {
//                 if let Err(e) = Self::process_event(&wallet, event).await {
//                     log::error!("Error processing wallet event: {}", e);
//                 }
//             }
//         });
//     }

//     /// Process a wallet event
//     async fn process_event(wallet: &Arc<WalletInterface>, event: WalletEvent) -> Result<(), WalletError> {
//         match event {
//             WalletEvent::BlockConnected(block) => {
//                 wallet.process_block(&block).await?;
//             }
//             WalletEvent::BlockDisconnected(_hash) => {
//                 // On block disconnection, we need to resync the wallet
//                 wallet.sync().await?;
//             }
//             WalletEvent::TransactionDetected(tx) => {
//                 wallet.process_transaction(&tx, None).await?;
//             }
//             WalletEvent::SyncRequested => {
//                 wallet.sync().await?;
//             }
//         }
//         Ok(())
//     }
// }
