use std::sync::Arc;
use tokio::net::UnixStream;

mod error;
mod chain;
mod connection;
mod notification;
mod generated;

pub use generated::*;
pub use error::BlockTalkError;
pub use connection::Connection;
pub use chain::ChainInterface;

#[derive(Clone)]
/// Main entry point for blockchain interaction
pub struct BlockTalk {
    connection: Arc<Connection>,
    chain: Arc<ChainInterface>,
}

impl BlockTalk {
    /// Initialize BlockTalk by connecting to a Bitcoin node through the specified socket
    pub async fn init(socket_path: &str) -> Result<Self, BlockTalkError> {
        // Establish connection to the node
        let connection = Connection::connect(socket_path).await?;
        
        // Create chain interface
        let chain = Arc::new(ChainInterface::new(connection.clone()));
        
        Ok(Self {
            connection,
            chain,
        })
    }
    
    /// Get a reference to the chain interface
    pub fn chain(&self) -> &Arc<ChainInterface> {
        &self.chain
    }
}