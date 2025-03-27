use bitcoin::hashes::Hash;
use bitcoin::{Transaction, Txid};
use std::sync::Arc;

use crate::{
    chain_capnp::chain::Client as ChainClient, proxy_capnp::thread::Client as ThreadClient,
    BlockTalkError,
};

#[derive(Debug)]
pub struct TransactionAncestry {
    /// Number of ancestor transactions
    pub ancestors: u64,
    /// Number of descendant transactions
    pub descendants: u64,
    /// Total size of ancestor transactions in bytes
    pub ancestor_size: u64,
    /// Total fees of ancestor transactions in satoshis
    pub ancestor_fees: i64,
}

#[async_trait::async_trait(?Send)]
pub trait MempoolInterface {
    /// Check if a transaction is in the mempool
    async fn is_in_mempool(&self, txid: &Txid) -> Result<bool, BlockTalkError>;

    /// Check if a transaction has descendants in the mempool
    async fn has_descendants_in_mempool(&self, txid: &Txid) -> Result<bool, BlockTalkError>;

    /// Broadcast a transaction to the network
    async fn broadcast_transaction(
        &self,
        tx: &Transaction,
        max_tx_fee: i64,
        relay: bool,
    ) -> Result<(String, bool), BlockTalkError>;

    /// Get transaction ancestry information
    async fn get_transaction_ancestry(
        &self,
        txid: &Txid,
    ) -> Result<TransactionAncestry, BlockTalkError>;
}

pub struct Mempool {
    chain_client: ChainClient,
    thread: ThreadClient,
}

#[async_trait::async_trait(?Send)]
impl MempoolInterface for Mempool {
    async fn is_in_mempool(&self, txid: &Txid) -> Result<bool, BlockTalkError> {
        log::debug!("Checking if transaction {} is in mempool", txid);
        let mut req = self.chain_client.is_in_mempool_request();

        req.get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get mempool context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        req.get().set_txid(txid.as_ref());

        let response = req.send().promise.await.map_err(|e| {
            log::error!("Failed to check mempool status: {}", e);
            BlockTalkError::Connection(e.to_string())
        })?;

        Ok(response.get()?.get_result())
    }

    async fn has_descendants_in_mempool(&self, txid: &Txid) -> Result<bool, BlockTalkError> {
        log::debug!(
            "Checking if transaction {} has descendants in mempool",
            txid
        );
        let mut req = self.chain_client.has_descendants_in_mempool_request();

        req.get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get mempool context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        req.get().set_txid(txid.as_ref());

        let response = req.send().promise.await.map_err(|e| {
            log::error!("Failed to check descendants: {}", e);
            BlockTalkError::Connection(e.to_string())
        })?;

        Ok(response.get()?.get_result())
    }

    async fn broadcast_transaction(
        &self,
        tx: &Transaction,
        max_tx_fee: i64,
        relay: bool,
    ) -> Result<(String, bool), BlockTalkError> {
        log::debug!("Broadcasting transaction {}", tx.compute_txid());
        let mut req = self.chain_client.broadcast_transaction_request();

        req.get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get broadcast context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        let mut params = req.get();
        params.set_tx(bitcoin::consensus::serialize(tx).as_slice());
        params.set_max_tx_fee(max_tx_fee);
        params.set_relay(relay);

        let response = req.send().promise.await.map_err(|e| {
            log::error!("Failed to broadcast transaction: {}", e);
            BlockTalkError::Connection(e.to_string())
        })?;

        let result = response.get()?;
        Ok((
            result
                .get_error()?
                .to_string()
                .map_err(|e| BlockTalkError::Connection(e.to_string()))?,
            result.get_result(),
        ))
    }

    async fn get_transaction_ancestry(
        &self,
        txid: &Txid,
    ) -> Result<TransactionAncestry, BlockTalkError> {
        log::debug!("Getting ancestry for transaction {}", txid);
        let mut req = self.chain_client.get_transaction_ancestry_request();

        req.get()
            .get_context()
            .map_err(|e| {
                log::error!("Failed to get ancestry context: {}", e);
                BlockTalkError::Connection(e.to_string())
            })?
            .set_thread(self.thread.clone());

        req.get().set_txid(txid.as_ref());

        let response = req.send().promise.await.map_err(|e| {
            log::error!("Failed to get transaction ancestry: {}", e);
            BlockTalkError::Connection(e.to_string())
        })?;

        let result = response.get()?;
        Ok(TransactionAncestry {
            ancestors: result.get_ancestors(),
            descendants: result.get_descendants(),
            ancestor_size: result.get_ancestorsize(),
            ancestor_fees: result.get_ancestorfees(),
        })
    }
}

impl Mempool {
    pub fn new(chain_client: ChainClient, thread: ThreadClient) -> Self {
        Self {
            chain_client,
            thread,
        }
    }
}
