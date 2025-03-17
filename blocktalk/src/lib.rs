use std::sync::Arc;

mod chain;
mod connection;
mod error;
mod generated;
mod notification;

pub use chain::ChainInterface;
pub use connection::{Connection, ConnectionProvider, UnixConnectionProvider};
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
        let connection = Connection::connect_default(socket_path).await?;
        let chain = Arc::new(ChainInterface::new(connection.clone()));
        log::info!("BlockTalk initialized successfully");

        Ok(Self { connection, chain })
    }

    pub async fn init_with_provider(
        socket_path: &str,
        provider: Box<dyn ConnectionProvider>,
    ) -> Result<Self, BlockTalkError> {
        log::info!(
            "Initializing BlockTalk with socket path: {} and custom provider",
            socket_path
        );
        let connection = Connection::connect(socket_path, provider).await?;
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
            Err(_) => Ok(()),
        }
    }
}
