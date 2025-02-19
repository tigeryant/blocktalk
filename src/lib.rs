use std::sync::Arc;

mod chain;
mod connection;
mod error;
mod generated;
mod notification;

pub use chain::ChainInterface;
pub use connection::Connection;
pub use error::BlockTalkError;
pub use generated::*;

#[derive(Clone)]
/// Main entry point for blockchain interaction
pub struct BlockTalk {
    connection: Arc<Connection>,
    chain: Arc<ChainInterface>,
}

impl BlockTalk {
    /// Initialize BlockTalk by connecting to a Bitcoin node through the specified socket
    pub async fn init(socket_path: &str) -> Result<Self, BlockTalkError> {
        log::info!("Initializing BlockTalk with socket path: {}", socket_path);
        let connection = Connection::connect(socket_path).await?;
        let chain = Arc::new(ChainInterface::new(connection.clone()));
        log::info!("BlockTalk initialized successfully");

        Ok(Self { connection, chain })
    }

    /// Get a reference to the chain interface
    pub fn chain(&self) -> &Arc<ChainInterface> {
        &self.chain
    }

    /// Disconnect from the node
    pub async fn disconnect(self) -> Result<(), BlockTalkError> {
        // Check if we're the last owner of the connection
        match Arc::try_unwrap(self.connection) {
            Ok(conn) => conn.disconnect().await,
            Err(_) => {
                // There are other references to the connection still alive
                Ok(())
            }
        }
    }
}
