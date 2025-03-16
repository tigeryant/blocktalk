use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::LocalSet;

use jsonrpc_core::IoHandler;
use jsonrpc_http_server::{Server, ServerBuilder};

use super::config::RpcConfig;
use super::handlers;
use crate::error::WalletError;
use crate::wallet::WalletInterface;

pub struct RPCServer {
    wallet: Arc<WalletInterface>,
    server: Option<Server>,
    config: RpcConfig,
}

impl RPCServer {
    pub fn new(wallet: Arc<WalletInterface>, config: &RpcConfig) -> Self {
        Self {
            wallet,
            server: None,
            config: config.clone(),
        }
    }

    pub async fn start(&mut self, bind_address: SocketAddr) -> Result<(), WalletError> {
        let wallet = self.wallet.clone();
        let mut io = IoHandler::new();

        handlers::register_wallet_methods(&mut io, wallet.clone());

        log::info!("Starting RPC server on {}", bind_address);
        let server = ServerBuilder::new(io)
            .threads(1) // Force single thread
            .start_http(&bind_address)
            .map_err(|e| WalletError::RPCError(format!("Failed to start RPC server: {}", e)))?;

        self.server = Some(server);
        log::info!("RPC server started");
        let local = LocalSet::new();
        local
            .run_until(async {
                loop {
                    tokio::task::yield_now().await;
                }
            })
            .await;

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(server) = self.server.take() {
            log::info!("Stopping RPC server");
            server.close();
            log::info!("RPC server stopped");
        }
    }
}
