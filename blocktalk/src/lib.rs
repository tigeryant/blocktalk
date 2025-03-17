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
pub use notification::ChainNotification;
pub use notification::NotificationHandler;

#[derive(Clone)]
pub struct BlockTalk {
    connection: Arc<Connection>,
    chain: Arc<ChainInterface>,
}

impl BlockTalk {
    pub async fn init(socket_path: &str) -> Result<Self, BlockTalkError> {
        log::info!("Initializing BlockTalk with socket path: {}", socket_path);
        let connection = Connection::connect(socket_path).await?;
        let chain = Arc::new(ChainInterface::new(connection.clone()));
        log::info!("BlockTalk initialized successfully");

        Ok(Self { connection, chain })
    }

    pub fn chain(&self) -> &Arc<ChainInterface> {
        &self.chain
    }

    pub async fn disconnect(self) -> Result<(), BlockTalkError> {
        match Arc::try_unwrap(self.connection) {
            Ok(conn) => conn.disconnect().await,
            Err(_) => {
                Ok(())
            }
        }
    }
}
