// use async_trait::async_trait;
// use bitcoin::{Block, Transaction};

// #[derive(Debug)]
// pub enum ChainNotification {
//     BlockConnected(Block),
//     BlockDisconnected(BlockHash),
//     TransactionAddedToMempool(Transaction),
//     TransactionRemovedFromMempool(bitcoin::Txid),
//     UpdatedBlockTip(BlockHash),
//     ChainStateFlushed,
// }

// #[async_trait]
// pub trait NotificationHandler: Send + Sync {
//     async fn handle_notification(&self, notification: ChainNotification) -> Result<(), BlockTalkError>;
// }

// impl chain_capnp::chain_notifications::Server for Arc<dyn NotificationHandler> {
//     fn destroy(
//         &mut self,
//         _: chain_capnp::chain_notifications::DestroyParams,
//         _: chain_capnp::chain_notifications::DestroyResults,
//     ) -> capnp::capability::Promise<(), capnp::Error> {
//         capnp::capability::Promise::ok(())
//     }

//     fn transaction_added_to_mempool(
//         &mut self,
//         params: chain_capnp::chain_notifications::TransactionAddedToMempoolParams,
//         _: chain_capnp::chain_notifications::TransactionAddedToMempoolResults,
//     ) -> capnp::capability::Promise<(), capnp::Error> {
//         let params = pry!(params.get());
//         let tx_data = pry!(params.get_tx());
//         let tx = match bitcoin::consensus::deserialize(tx_data) {
//             Ok(tx) => tx,
//             Err(_) => return capnp::capability::Promise::err(
//                 capnp::Error::failed("Invalid transaction data".to_string())
//             ),
//         };
        
//         let handler = self.clone();
//         capnp::capability::Promise::from_future(async move {
//             handler.handle_notification(ChainNotification::TransactionAddedToMempool(tx)).await
//                 .map_err(|e| capnp::Error::failed(e.to_string()))?;
//             Ok(())
//         })
//     }
// }