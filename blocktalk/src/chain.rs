use bitcoin::consensus::Decodable;
use bitcoin::hashes::Hash;
use bitcoin::{Block, BlockHash};
use std::sync::Arc;
use std::sync::Mutex;

use crate::error::ChainErrorKind;
use crate::{
    chain_capnp::chain::Client as ChainClient,
    notification::{ChainNotificationHandler, NotificationHandler},
    proxy_capnp::thread::Client as ThreadClient,
    BlockTalkError, Connection,
};

#[async_trait::async_trait(?Send)]
pub trait ChainInterface {
    /// Get the current tip block's height and hash
    async fn get_tip(&self) -> Result<(i32, BlockHash), BlockTalkError>;

    /// Get the timestamp of the current chain tip
    async fn tip_time(&self) -> Result<u32, BlockTalkError>;

    /// Get a block at a specific height
    async fn get_block(
        &self,
        node_tip_hash: &bitcoin::BlockHash,
        height: i32,
    ) -> Result<Block, BlockTalkError>;

    /// Get the genesis block (block at height 0)
    async fn get_genesis_block(&self) -> Result<Block, BlockTalkError>;

    /// Check if the node is fully synced
    /// Returns true if the node is fully synced, false if it's still in initial block download
    async fn is_synced(&self) -> Result<bool, BlockTalkError>;

    /// Check if a block is in the best chain
    async fn is_in_best_chain(&self, block_hash: &BlockHash) -> Result<bool, BlockTalkError>;

    /// Find the common ancestor between two blocks
    async fn find_common_ancestor(
        &self,
        block1_hash: &BlockHash,
        block2_hash: &BlockHash,
    ) -> Result<Option<BlockHash>, BlockTalkError>;

    /// Get a full block by its hash
    async fn get_block_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> Result<Option<Block>, BlockTalkError>;

    /// Add a notification handler to receive chain updates
    async fn add_notification_handler(
        &self,
        handler: Arc<dyn NotificationHandler>,
    ) -> Result<(), BlockTalkError>;

    /// Remove a previously added notification handler
    async fn remove_notification_handler(
        &self,
        handler: Arc<dyn NotificationHandler>,
    ) -> Result<(), BlockTalkError>;

    /// Start receiving chain updates
    /// This must be called after adding handlers for them to receive updates
    async fn begin_chain_updates(&self) -> Result<(), BlockTalkError>;

    /// Stop receiving chain updates
    /// Handlers will stop receiving updates but remain registered
    async fn stop_chain_updates(&self) -> Result<(), BlockTalkError>;
}

pub struct Blockchain {
    chain_client: ChainClient,
    thread: ThreadClient,
    notification_handler: Arc<Mutex<ChainNotificationHandler>>,
}

#[async_trait::async_trait(?Send)]
impl ChainInterface for Blockchain {
    async fn get_tip(&self) -> Result<(i32, BlockHash), BlockTalkError> {
        log::debug!("Fetching current chain tip");
        let height = {
            let mut height_req = self.chain_client.get_height_request();
            height_req
                .get()
                .get_context()
                .map_err(|e| {
                    log::error!("Failed to get height context: {}", e);
                    BlockTalkError::Connection(e.to_string())
                })?
                .set_thread(self.thread.clone());

            let response = height_req.send().promise.await.map_err(|e| {
                log::error!("Failed to get chain height: {}", e);
                BlockTalkError::chain_error(ChainErrorKind::InvalidHeight, e.to_string())
            })?;
            response.get()?.get_result()
        };

        let hash_bytes = {
            let mut hash_req = self.chain_client.get_block_hash_request();
            hash_req
                .get()
                .get_context()
                .map_err(|e| {
                    log::error!("Failed to get block hash context: {}", e);
                    BlockTalkError::Connection(e.to_string())
                })?
                .set_thread(self.thread.clone());

            hash_req.get().set_height(height);
            let response = hash_req.send().promise.await.map_err(|e| {
                log::error!("Failed to get block hash at height {}: {}", height, e);
                BlockTalkError::chain_error(ChainErrorKind::BlockNotFound, e.to_string())
            })?;
            response.get()?.get_result()?.to_vec()
        };

        let hash = self.bytes_to_block_hash(&hash_bytes).map_err(|e| {
            log::error!("Failed to convert hash bytes to BlockHash: {}", e);
            e
        })?;

        log::debug!(
            "Retrieved chain tip at height {} with hash {}",
            height,
            hash
        );
        Ok((height, hash))
    }

    async fn tip_time(&self) -> Result<u32, BlockTalkError> {
        log::debug!("Fetching chain tip timestamp");
        let (_, tip_hash) = self.get_tip().await?;
        
        let block = self.get_block_by_hash(&tip_hash).await?
            .ok_or_else(|| BlockTalkError::chain_error(ChainErrorKind::BlockNotFound, "Tip block not found".to_string()))?;
        
        let timestamp = block.header.time;
        log::debug!("Chain tip timestamp: {}", timestamp);
        Ok(timestamp)
    }

    async fn get_block(
        &self,
        node_tip_hash: &bitcoin::BlockHash,
        height: i32,
    ) -> Result<Block, BlockTalkError> {
        log::debug!("Getting block at height {}", height);
        let mut find_req = self.chain_client.find_ancestor_by_height_request();

        find_req
            .get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get block context at height {}: {}", height, e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        let mut params = find_req.get();
        params.set_block_hash(node_tip_hash.as_ref());
        params.set_ancestor_height(height);
        params
            .get_ancestor()
            .map_err(|e| {
                log::error!(
                    "Failed to set ancestor parameters at height {}: {}",
                    height,
                    e
                );
                BlockTalkError::chain_error(ChainErrorKind::InvalidAncestor, e.to_string())
            })?
            .set_want_data(true);

        let response = find_req.send().promise.await.map_err(|e| {
            log::error!("Failed to fetch block at height {}: {}", height, e);
            BlockTalkError::chain_error(ChainErrorKind::BlockNotFound, e.to_string())
        })?;

        let mut data = response.get()?.get_ancestor()?.get_data()?;

        Block::consensus_decode(&mut data).map_err(|e| {
            log::error!("Failed to decode block at height {}: {}", height, e);
            BlockTalkError::chain_error(ChainErrorKind::DeserializationFailed, e.to_string())
        })
    }

    async fn get_genesis_block(&self) -> Result<Block, BlockTalkError> {
        log::debug!("Fetching genesis block");
        let (_, tip_hash) = self.get_tip().await?;
        self.get_block(&tip_hash, 0).await
    }

    async fn is_synced(&self) -> Result<bool, BlockTalkError> {
        log::debug!("Checking sync status");
        
        let mut ibd_req = self.chain_client.is_initial_block_download_request();
        ibd_req
            .get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get IBD context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        let ibd_response = ibd_req.send().promise.await.map_err(|e| {
            log::error!("Failed to check IBD status: {}", e);
            BlockTalkError::Connection(e.to_string())
        })?;

        let is_ibd = ibd_response.get()?.get_result();
        log::debug!("IBD result value: {}", is_ibd);
        Ok(!is_ibd)
    }

    async fn is_in_best_chain(&self, block_hash: &BlockHash) -> Result<bool, BlockTalkError> {
        log::debug!("Checking if block {} is in best chain", block_hash);
        let hash_bytes = block_hash.to_raw_hash().to_byte_array();

        let mut find_req = self.chain_client.find_block_request();
        find_req
            .get()
            .get_context()?
            .set_thread(self.thread.clone());
        find_req.get().set_hash(&hash_bytes);

        let response = find_req.send().promise.await.map_err(|e| {
            log::error!("Failed to find block {}: {}", block_hash, e);
            BlockTalkError::chain_error(ChainErrorKind::BlockNotFound, e.to_string())
        })?;

        let block_info = response.get()?.get_block().map_err(|e| {
            log::error!("Failed to get block info for {}: {}", block_hash, e);
            BlockTalkError::chain_error(ChainErrorKind::InvalidBlockData, e.to_string())
        })?;

        let is_active = block_info.get_in_active_chain() != 0;

        log::debug!(
            "Block {} is {} in the active chain",
            block_hash,
            if is_active {
                "included"
            } else {
                "not included"
            }
        );
        Ok(is_active)
    }

    async fn find_common_ancestor(
        &self,
        block1_hash: &BlockHash,
        block2_hash: &BlockHash,
    ) -> Result<Option<BlockHash>, BlockTalkError> {
        log::debug!(
            "Finding common ancestor between blocks {} and {}",
            block1_hash,
            block2_hash
        );
        let hash1_bytes = block1_hash.to_raw_hash().to_byte_array();
        let hash2_bytes = block2_hash.to_raw_hash().to_byte_array();

        let mut find_req = self.chain_client.find_common_ancestor_request();
        find_req
            .get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get ancestor context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        {
            let mut params = find_req.get();
            params.set_block_hash1(&hash1_bytes);
            params.set_block_hash2(&hash2_bytes);
        }

        let response = find_req.send().promise.await.map_err(|e| {
            log::error!("Failed to find common ancestor: {}", e);
            BlockTalkError::chain_error(ChainErrorKind::InvalidAncestor, e.to_string())
        })?;

        let ancestor_bytes = response.get()?.get_ancestor()?.get_data()?;
        if ancestor_bytes.is_empty() {
            log::debug!("No common ancestor found");
            Ok(None)
        } else {
            let ancestor_hash = self.bytes_to_block_hash(ancestor_bytes)?;
            log::debug!("Common ancestor found: {}", ancestor_hash);
            Ok(Some(ancestor_hash))
        }
    }

    async fn get_block_by_hash(
        &self,
        block_hash: &BlockHash,
    ) -> Result<Option<Block>, BlockTalkError> {
        log::debug!("Getting block with hash {}", block_hash);
        let hash_bytes = block_hash.to_raw_hash().to_byte_array();

        let mut find_req = self.chain_client.find_block_request();
        find_req
            .get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get block context for hash {}: {}", block_hash, e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        find_req.get().set_hash(&hash_bytes);
        let response = find_req.send().promise.await.map_err(|e| {
            log::error!("Failed to fetch block with hash {}: {}", block_hash, e);
            BlockTalkError::chain_error(ChainErrorKind::BlockNotFound, e.to_string())
        })?;

        let block_info = response.get()?.get_block()?;
        if !block_info.has_data() || block_info.get_data()?.is_empty() {
            log::debug!("No block data found for hash {}", block_hash);
            return Ok(None);
        }

        match bitcoin::consensus::deserialize::<Block>(block_info.get_data()?) {
            Ok(block) => {
                log::debug!("Successfully retrieved block {}", block_hash);
                Ok(Some(block))
            }
            Err(e) => {
                log::error!("Failed to deserialize block {}: {}", block_hash, e);
                Err(BlockTalkError::chain_error(
                    ChainErrorKind::DeserializationFailed,
                    e.to_string(),
                ))
            }
        }
    }

    async fn add_notification_handler(
        &self,
        handler: Arc<dyn NotificationHandler>,
    ) -> Result<(), BlockTalkError> {
        let mut notification_handler = self.notification_handler.lock().map_err(|e| {
            BlockTalkError::Connection(format!(
                "Failed to acquire lock for notification handler: {}",
                e
            ))
        })?;
        notification_handler.register_handler(handler).await
    }

    async fn remove_notification_handler(
        &self,
        handler: Arc<dyn NotificationHandler>,
    ) -> Result<(), BlockTalkError> {
        let mut notification_handler = self.notification_handler.lock().map_err(|e| {
            BlockTalkError::Connection(format!(
                "Failed to acquire lock for notification handler: {}",
                e
            ))
        })?;
        // TODO: Implement handler removal in ChainNotificationHandler if possible
        Ok(())
    }

    async fn begin_chain_updates(&self) -> Result<(), BlockTalkError> {
        log::debug!("Starting chain update notifications");
        let handler = self.notification_handler.lock().unwrap().clone();
        let notification_client = capnp_rpc::new_client(handler);
        let mut handle_req = self.chain_client.handle_notifications_request();

        handle_req
            .get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get notification context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        handle_req.get().set_notifications(notification_client);
        handle_req.send().promise.await.map_err(|e| {
            log::error!("Failed to start chain updates: {}", e);
            BlockTalkError::Connection(e.to_string())
        })?;

        log::info!("Successfully started chain updates");
        Ok(())
    }

    async fn stop_chain_updates(&self) -> Result<(), BlockTalkError> {
        // TODO: Implement stopping notifications in the Cap'n Proto RPC layer
        Ok(())
    }
}

impl Blockchain {
    pub fn new(connection: Arc<Connection>) -> Self {
        Self {
            chain_client: connection.chain_client().clone(),
            thread: connection.thread().clone(),
            notification_handler: Arc::new(Mutex::new(ChainNotificationHandler::new())),
        }
    }

    pub fn from_client(chain_client: ChainClient, thread: ThreadClient) -> Self {
        Self {
            chain_client,
            thread,
            notification_handler: Arc::new(Mutex::new(ChainNotificationHandler::new())),
        }
    }

    pub fn notification_handler(&self) -> Arc<Mutex<ChainNotificationHandler>> {
        self.notification_handler.clone()
    }

    // Helper method to convert bytes to BlockHash
    fn bytes_to_block_hash(&self, bytes: &[u8]) -> Result<BlockHash, BlockTalkError> {
        if bytes.len() != 32 {
            log::error!("Invalid hash length: expected 32, got {}", bytes.len());
            return Err(BlockTalkError::chain_error(
                ChainErrorKind::InvalidBlockData,
                format!("Invalid hash length: expected 32, got {}", bytes.len()),
            ));
        }

        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(bytes);
        Ok(BlockHash::from_raw_hash(
            bitcoin::hashes::Hash::from_byte_array(hash_bytes),
        ))
    }
}
