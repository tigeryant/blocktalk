use bitcoin::{Block, BlockHash, Transaction, Txid, consensus::Decodable};
use async_trait::async_trait;
use crate::error::BlockTalkError;

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
    async fn handle_notification(&self, notification: ChainNotification) -> Result<(), BlockTalkError>;
}

pub struct ChainNotificationHandler {
    handlers: Vec<Box<dyn NotificationHandler>>,
}

impl ChainNotificationHandler {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    pub fn register_handler(&mut self, handler: Box<dyn NotificationHandler>) {
        self.handlers.push(handler);
    }

    async fn dispatch_notification(&self, notification: ChainNotification) -> Result<(), BlockTalkError> {
        for handler in &self.handlers {
            handler.handle_notification(notification.clone()).await?;
        }
        Ok(())
    }
}

// // Private implementation details
// #[doc(hidden)]
// mod internal {
//     use super::*;
//     use crate::chain_capnp::chain_notifications::*;
    
//     #[async_trait]
//     pub trait CapnpNotificationHandler {
//         async fn handle_block_connected(
//             &self,
//             params: BlockConnectedParams,
//         ) -> Result<(), BlockTalkError>;

//         async fn handle_block_disconnected(
//             &self,
//             params: BlockDisconnectedParams,
//         ) -> Result<(), BlockTalkError>;

//         async fn handle_transaction_added(
//             &self,
//             params: TransactionAddedToMempoolParams,
//         ) -> Result<(), BlockTalkError>;

//         async fn handle_transaction_removed(
//             &self,
//             params: TransactionRemovedFromMempoolParams,
//         ) -> Result<(), BlockTalkError>;

//         async fn handle_updated_block_tip(
//             &self,
//             params: UpdatedBlockTipParams,
//         ) -> Result<(), BlockTalkError>;

//         async fn handle_chain_state_flushed(
//             &self
//         ) -> Result<(), BlockTalkError>;
//     }

//     #[async_trait(?Send)]
//     impl CapnpNotificationHandler for ChainNotificationHandler {
//         async fn handle_block_connected(
//             &self,
//             params: BlockConnectedParams,
//         ) -> Result<(), BlockTalkError> {
//             let info = params.get()?.get_block()?;
//             let block = Block::consensus_decode(&mut info.get_data()?)
//                 .map_err(|_| BlockTalkError::InvalidBlockData)?;
//             self.dispatch_notification(ChainNotification::BlockConnected(block)).await
//         }

//         async fn handle_block_disconnected(
//             &self,
//             params: BlockDisconnectedParams,
//         ) -> Result<(), BlockTalkError> {
//             let info = params.get()?.get_block()?;
//             let hash = BlockHash::consensus_decode(&mut info.get_data()?)
//                 .map_err(|_| BlockTalkError::InvalidBlockData)?;
//             self.dispatch_notification(ChainNotification::BlockDisconnected(hash)).await
//         }

//         async fn handle_transaction_added(
//             &self,
//             params: TransactionAddedToMempoolParams,
//         ) -> Result<(), BlockTalkError> {
//             let info = params.get()?.get_tx()?;
//             let tx = Transaction::consensus_decode(&mut info.get_da)
//                 .map_err(|_| BlockTalkError::InvalidBlockData)?;
//             self.dispatch_notification(ChainNotification::TransactionAddedToMempool(tx)).await
//         }

//         async fn handle_transaction_removed(
//             &self,
//             params: TransactionRemovedFromMempoolParams,
//         ) -> Result<(), BlockTalkError> {
//             let info = params.get()?.get_transaction()?;
//             let txid = Txid::consensus_decode(&mut info.get_data()?)
//                 .map_err(|_| BlockTalkError::InvalidBlockData)?;
//             self.dispatch_notification(ChainNotification::TransactionRemovedFromMempool(txid)).await
//         }

//         async fn handle_updated_block_tip(
//             &self,
//             params: UpdatedBlockTipParams,
//         ) -> Result<(), BlockTalkError> {
//             let info = params.get()?.get_block()?;
//             let hash = BlockHash::consensus_decode(&mut info.get_data()?)
//                 .map_err(|_| BlockTalkError::InvalidBlockData)?;
//             self.dispatch_notification(ChainNotification::UpdatedBlockTip(hash)).await
//         }

//         async fn handle_chain_state_flushed(
//             &self,
//         ) -> Result<(), BlockTalkError> {
//             self.dispatch_notification(ChainNotification::ChainStateFlushed).await
//         }
//     }

//     // Bridge between Cap'n Proto and our internal handlers
//     pub(crate) async fn handle_capnp_notification<T, F, R>(
//         handler: &ChainNotificationHandler,
//         params: T,
//         handler_fn: F,
//         results: capnp::capability::Response<R>,
//     ) -> Result<(), capnp::Error> 
//     where
//         T: Send + 'static,
//         F: FnOnce(&ChainNotificationHandler, T) -> Pin<Box<dyn Future<Output = Result<(), BlockTalkError>> + Send>> + Send,
//         R: capnp::capability::Server,
//     {
//         handler_fn(handler, params).await
//             .map_err(|e| capnp::Error::failed(format!("Failed to handle notification: {}", e)))?;
//         Ok(R::new(results))
//     }
// }

// // Re-export only what's needed for the public API
// pub use self::internal::CapnpNotificationHandler;