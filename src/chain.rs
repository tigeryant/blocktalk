use crate::{
    chain_capnp::chain::Client as ChainClient,
    BlockTalkError,
    Connection,
    notification::{ChainNotificationHandler, NotificationHandler},
};
use std::sync::Arc;

pub struct ChainInterface {
    chain_client: ChainClient,
    notification_handler: ChainNotificationHandler,
}

impl ChainInterface {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self {
            chain_client: connection.chain_client().clone(),
            notification_handler: ChainNotificationHandler::new(),
        }
    }

    pub fn from_client(chain_client: ChainClient) -> Self {
        Self {
            chain_client,
            notification_handler: ChainNotificationHandler::new(),
        }
    }

    pub fn register_handler(&mut self, handler: Box<dyn NotificationHandler>) {
        self.notification_handler.register_handler(handler);
    }

    pub fn notification_handler(&self) -> &ChainNotificationHandler {
        &self.notification_handler
    }

    /// Get the current tip block's height and hash
    pub async fn get_tip(&self) -> Result<(i32, Vec<u8>), BlockTalkError> {
        let height = {
            println!("get_tip: Building height request");
            let request = self.chain_client.get_height_request();
            println!("get_tip: Built height request");
            
            println!("get_tip: Sending height request");
            let promise = request.send().promise;
            println!("get_tip: Request sent, awaiting response");
            
            let response = promise.await?;
            println!("get_tip: Received height response");
            
            let height_result = response.get()?;
            println!("get_tip: Parsed height response");
            height_result.get_result()
        };

        println!("Height: {}", height);

        let hash = {
            let mut request = self.chain_client.get_block_hash_request();
            request.get().set_height(height);
            let response = request.send().promise.await?;
            response.get()?.get_result()?.to_vec()
        };

        println!("hash: {:?}", hash);

        Ok((height, hash))
    }

    /// Get block hash at specific height
    pub async fn get_block_at_height(
        &self,
        height: i32,
    ) -> Result<Option<Vec<u8>>, BlockTalkError> {
        let mut request = self.chain_client.get_block_hash_request();
        request.get().set_height(height);
        let response = request.send().promise.await?;

        // If block doesn't exist at this height, return None
        if response.get()?.get_result()?.is_empty() {
            return Ok(None);
        }

        Ok(Some(response.get()?.get_result()?.to_vec()))
    }

    /// Check if a block is in the best chain
    pub async fn is_in_best_chain(&self, block_hash: &[u8]) -> Result<bool, BlockTalkError> {
        let mut request = self.chain_client.find_block_request();
        request.get().set_hash(block_hash);
        let response = request.send().promise.await?;
        let block_info = response.get()?.get_block()?;

        Ok(block_info.get_in_active_chain() != 0)
    }

    /// Find the common ancestor between two blocks
    pub async fn find_common_ancestor(
        &self,
        block1_hash: &[u8],
        block2_hash: &[u8],
    ) -> Result<Option<Vec<u8>>, BlockTalkError> {
        let mut request = self.chain_client.find_common_ancestor_request();
        {
            let mut params = request.get();
            params.set_block_hash1(block1_hash);
            params.set_block_hash2(block2_hash);
        }
        let response = request.send().promise.await?;

        let ancestor = response.get()?.get_ancestor()?.get_data()?;
        if ancestor.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ancestor.to_vec()))
        }
    }
}