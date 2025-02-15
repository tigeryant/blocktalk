use crate::{chain_capnp::chain::Client as ChainClient, BlockTalkError, Connection};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Represents a block notification event
#[derive(Clone, Debug)]
pub struct BlockNotification {
    pub height: u32,
    pub hash: Vec<u8>,
    pub time: u64,
}

/// High-level interface for interacting with the blockchain
pub struct ChainInterface {
    chain_client: ChainClient,
    notification_publisher: broadcast::Sender<BlockNotification>,
}

impl ChainInterface {
    /// Create a new ChainInterface from an existing Connection
    pub fn new(connection: Arc<Connection>) -> Self {
        let (notification_publisher, _) = broadcast::channel(100);
        Self {
            chain_client: connection.chain_client().clone(),
            notification_publisher,
        }
    }

    /// Create a new ChainInterface from just a ChainClient
    pub fn from_client(chain_client: ChainClient) -> Self {
        let (notification_publisher, _) = broadcast::channel(100);
        Self {
            chain_client,
            notification_publisher,
        }
    }

    /// Get the current tip block's height and hash
    pub async fn get_tip(&self) -> Result<(i32, Vec<u8>), BlockTalkError> {
        let height = {
            let request = self.chain_client.get_height_request();
            let response = request.send().promise.await?;
            let height_result = response.get()?;
            height_result.get_result()
        };

        let hash = {
            let mut request = self.chain_client.get_block_hash_request();
            request.get().set_height(height);
            let response = request.send().promise.await?;
            response.get()?.get_result()?.to_vec()
        };

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

    /// Subscribe to block notifications
    pub fn subscribe(&self) -> broadcast::Receiver<BlockNotification> {
        self.notification_publisher.subscribe()
    }
}
