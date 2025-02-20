use async_trait::async_trait;
use bitcoin::{Block, BlockHash, Transaction, Txid, consensus::Decodable};
use std::sync::Arc;
use tokio::sync::Mutex;
use bitcoin::hashes::{Hash};

use crate::error::BlockTalkError;
use crate::chain_capnp::chain_notifications;

// Public interface
#[derive(Clone, Debug)]
pub enum ChainNotification {
    BlockConnected(Block),
    BlockDisconnected(BlockHash),
    TransactionAddedToMempool(Transaction),
    TransactionRemovedFromMempool(Txid),
    UpdatedBlockTip(BlockHash),
    ChainStateFlushed,
}

#[async_trait]
pub trait NotificationHandler: Send + Sync {
    async fn handle_notification(
        &self,
        notification: ChainNotification,
    ) -> Result<(), BlockTalkError>;
}

#[derive(Clone)]
pub struct ChainNotificationHandler {
    handlers: Arc<Mutex<Vec<Arc<dyn NotificationHandler>>>>,
}

impl ChainNotificationHandler {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn register_handler(&mut self, handler: Arc<dyn NotificationHandler>) {
        let mut guard = self.handlers.lock().await;
        guard.push(handler);
    }

    async fn dispatch_notification(
        &self,
        notification: ChainNotification,
    ) -> Result<(), BlockTalkError> {
        // Get a copy of the handlers to avoid holding the lock during async calls
        let handlers = {
            let guard = self.handlers.lock().await;
            guard.clone()
        };
        
        for handler in handlers {
            handler.handle_notification(notification.clone()).await?;
        }
        Ok(())
    }
}

impl Default for ChainNotificationHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl chain_notifications::Server for ChainNotificationHandler {
    fn block_connected(
        &mut self,
        params: chain_notifications::BlockConnectedParams,
        _: chain_notifications::BlockConnectedResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        let handler = self.clone();
        
        // Create a future that returns Result<(), Error>
        let future = async move {
            // Get block info using POC pattern
            let params_reader = params.get()?;
            let block_info = params_reader.get_block()?;
            let mut block_data = block_info.get_data()?;
            
            // Decode the block
            let block = bitcoin::Block::consensus_decode(&mut block_data)
                .map_err(|e| ::capnp::Error::failed(format!("Failed to decode block: {}", e)))?;
            
            // Dispatch notification
            handler.dispatch_notification(ChainNotification::BlockConnected(block)).await
                .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
        };
        
        // Convert the future to a Promise
        ::capnp::capability::Promise::from_future(future)
    }

    fn block_disconnected(
        &mut self,
        params: chain_notifications::BlockDisconnectedParams,
        _: chain_notifications::BlockDisconnectedResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        let handler = self.clone();
        
        let future = async move {
            // Get block info using POC pattern
            let params_reader = params.get()?;
            let block_info = params_reader.get_block()?;
            
            // Get height
            let height = block_info.get_height();
            
            // Get hash using POC pattern
            let hash_data = block_info.get_hash()?;
            
            // Create BlockHash from sha256d hash
            let hash = {
                let hash_obj = bitcoin::hashes::sha256d::Hash::from_slice(hash_data)
                    .map_err(|e| ::capnp::Error::failed(format!("Invalid block hash: {}", e)))?;
                bitcoin::BlockHash::from(hash_obj)
            };
            
            // Dispatch notification
            handler.dispatch_notification(ChainNotification::BlockDisconnected(hash)).await
                .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
        };
        
        ::capnp::capability::Promise::from_future(future)
    }

    // fn transaction_added_to_mempool(
    //     &mut self,
    //     params: chain_notifications::TransactionAddedToMempoolParams,
    //     _: chain_notifications::TransactionAddedToMempoolResults,
    // ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
    //     let handler = self.clone();
        
    //     let future = async move {
    //         // Get transaction data
    //         let tx_reader = params.get()?;
    //         let tx_data = tx_reader.get_data()?;
            
    //         // Decode transaction
    //         let tx = bitcoin::Transaction::consensus_decode(&mut tx_data)
    //             .map_err(|e| ::capnp::Error::failed(format!("Failed to decode transaction: {}", e)))?;
            
    //         // Dispatch notification
    //         handler.dispatch_notification(ChainNotification::TransactionAddedToMempool(tx)).await
    //             .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
    //     };
        
    //     ::capnp::capability::Promise::from_future(future)
    // }

    // fn transaction_removed_from_mempool(
    //     &mut self,
    //     params: chain_notifications::TransactionRemovedFromMempoolParams,
    //     _: chain_notifications::TransactionRemovedFromMempoolResults,
    // ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
    //     let handler = self.clone();
        
    //     let future = async move {
    //         // Get txid
    //         let params_reader = params.get()?;
    //         let txid_reader = params_reader.get_txid()?;
    //         let txid_data = txid_reader.as_bytes()?;
            
    //         // Decode txid
    //         let txid = bitcoin::Txid::consensus_decode(&mut txid_data)
    //             .map_err(|e| ::capnp::Error::failed(format!("Failed to decode txid: {}", e)))?;
            
    //         // Dispatch notification
    //         handler.dispatch_notification(ChainNotification::TransactionRemovedFromMempool(txid)).await
    //             .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
    //     };
        
    //     ::capnp::capability::Promise::from_future(future)
    // }

    // fn updated_block_tip(
    //     &mut self,
    //     params: chain_notifications::UpdatedBlockTipParams,
    //     _: chain_notifications::UpdatedBlockTipResults,
    // ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
    //     let handler = self.clone();
        
    //     let future = async move {
    //         // Get block hash
    //         let params_reader = params.get()?;
    //         // Try getting a block field first, then the hash from it
    //         let block_info = params_reader.get_block()?;
    //         let hash_data = block_info.get_hash()?;
            
    //         // Create BlockHash from sha256d hash
    //         let hash = {
    //             let hash_obj = bitcoin::hashes::sha256d::Hash::from_slice(hash_data)
    //                 .map_err(|e| ::capnp::Error::failed(format!("Invalid block hash: {}", e)))?;
    //             bitcoin::BlockHash::from(hash_obj)
    //         };
            
    //         // Dispatch notification
    //         handler.dispatch_notification(ChainNotification::UpdatedBlockTip(hash)).await
    //             .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
    //     };
        
    //     ::capnp::capability::Promise::from_future(future)
    // }

    fn updated_block_tip(
        &mut self,
        _params: chain_notifications::UpdatedBlockTipParams,
        _: chain_notifications::UpdatedBlockTipResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        let handler = self.clone();
        
        let future = async move {
            // Simply log that we received the notification
            log::info!("Block tip updated - details skipped");
            
            // Create a dummy block hash - in a real implementation you'd get this from params
            let dummy_hash = bitcoin::BlockHash::all_zeros();
            
            // Dispatch notification with dummy data
            handler.dispatch_notification(ChainNotification::UpdatedBlockTip(dummy_hash)).await
                .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
        };
        
        ::capnp::capability::Promise::from_future(future)
    }

    fn chain_state_flushed(
        &mut self,
        _params: chain_notifications::ChainStateFlushedParams,
        _: chain_notifications::ChainStateFlushedResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        let handler = self.clone();
        
        let future = async move {
            // Dispatch notification
            handler.dispatch_notification(ChainNotification::ChainStateFlushed).await
                .map_err(|e| ::capnp::Error::failed(format!("Failed to dispatch notification: {}", e)))
        };
        
        ::capnp::capability::Promise::from_future(future)
    }

    fn destroy(
        &mut self,
        _params: chain_notifications::DestroyParams,
        _: chain_notifications::DestroyResults,
    ) -> ::capnp::capability::Promise<(), ::capnp::Error> {
        ::capnp::capability::Promise::ok(())
    }
}