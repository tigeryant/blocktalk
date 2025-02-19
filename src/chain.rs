use std::sync::Arc;

use crate::{
    chain_capnp::chain::Client as ChainClient,
    notification::{ChainNotificationHandler, NotificationHandler},
    proxy_capnp::thread::Client as ThreadClient,
    BlockTalkError, Connection,
};

pub struct ChainInterface {
    chain_client: ChainClient,
    thread: ThreadClient,
    notification_handler: ChainNotificationHandler,
}

impl ChainInterface {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self {
            chain_client: connection.chain_client().clone(),
            thread: connection.thread().clone(),
            notification_handler: ChainNotificationHandler::new(),
        }
    }

    pub fn from_client(chain_client: ChainClient, thread: ThreadClient) -> Self {
        Self {
            chain_client,
            thread,
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
        log::debug!("Fetching current chain tip");
        let height = {
            log::trace!("Sending height request");
            let mut height_req = self.chain_client.get_height_request();
            height_req
                .get()
                .get_context()?
                .set_thread(self.thread.clone());
            let promise = height_req.send().promise;
            let response = promise.await?;
            let height_result = response.get()?;
            height_result.get_result()
        };

        let hash = {
            let mut hash_req = self.chain_client.get_block_hash_request();
            hash_req
                .get()
                .get_context()?
                .set_thread(self.thread.clone());
            hash_req.get().set_height(height);
            let response = hash_req.send().promise.await?;
            response.get()?.get_result()?.to_vec()
        };

        log::trace!("Chain tip height: {}", height);
        log::trace!("Chain tip hash: {:?}", hash);
        log::debug!(
            "Retrieved chain tip at height {} with hash of {} bytes",
            height,
            hash.len()
        );

        Ok((height, hash))
    }

    /// Get block hash at specific height
    pub async fn get_block_at_height(
        &self,
        height: i32,
    ) -> Result<Option<Vec<u8>>, BlockTalkError> {
        log::debug!("Getting block hash at height {}", height);
        let mut hash_req = self.chain_client.get_block_hash_request();
        hash_req
            .get()
            .get_context()?
            .set_thread(self.thread.clone());
        hash_req.get().set_height(height);
        let response = hash_req.send().promise.await?;

        // If block doesn't exist at this height, return None
        if response.get()?.get_result()?.is_empty() {
            log::debug!("No block found at height {}", height);
            return Ok(None);
        }

        let hash = response.get()?.get_result()?.to_vec();
        log::debug!("Retrieved block hash at height {}", height);
        log::trace!("Block hash: {:?}", hash);

        Ok(Some(hash))
    }

    /// Check if a block is in the best chain
    pub async fn is_in_best_chain(&self, block_hash: &[u8]) -> Result<bool, BlockTalkError> {
        log::debug!("Checking if block is in best chain");
        let mut find_req = self.chain_client.find_block_request();
        find_req
            .get()
            .get_context()?
            .set_thread(self.thread.clone());
        find_req.get().set_hash(block_hash);
        let response = find_req.send().promise.await?;
        let block_info = response.get()?.get_block()?;
        let is_active = block_info.get_in_active_chain() != 0;

        log::debug!(
            "Block is {} in the active chain",
            if is_active {
                "included"
            } else {
                "not included"
            }
        );
        Ok(is_active)
    }

    /// Find the common ancestor between two blocks
    pub async fn find_common_ancestor(
        &self,
        block1_hash: &[u8],
        block2_hash: &[u8],
    ) -> Result<Option<Vec<u8>>, BlockTalkError> {
        log::debug!("Finding common ancestor between two blocks");
        let mut find_req = self.chain_client.find_common_ancestor_request();
        find_req
            .get()
            .get_context()?
            .set_thread(self.thread.clone());
        {
            let mut params = find_req.get();
            params.set_block_hash1(block1_hash);
            params.set_block_hash2(block2_hash);
        }
        let response = find_req.send().promise.await?;

        let ancestor = response.get()?.get_ancestor()?.get_data()?;
        if ancestor.is_empty() {
            log::debug!("No common ancestor found");
            Ok(None)
        } else {
            log::debug!("Common ancestor found");
            log::trace!("Ancestor hash: {:?}", ancestor);
            Ok(Some(ancestor.to_vec()))
        }
    }
}
