use std::net::SocketAddr;
use std::sync::Arc;

use jsonrpc_core::IoHandler;
use jsonrpc_http_server::{Server, ServerBuilder};

use crate::error::WalletError;
use crate::wallet::WalletInterface;
use super::handlers;
use super::config::RpcConfig;

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
    
    /// Start the RPC server
    pub async fn start(&mut self, bind_address: SocketAddr) -> Result<(), WalletError> {
        let wallet = self.wallet.clone();
        let mut io = IoHandler::new();
        
        // Register wallet methods
        handlers::register_wallet_methods(&mut io, wallet.clone());
        
        // Register blockchain methods (if implemented)
        // handlers::register_blockchain_methods(&mut io, wallet.clone());
        
        // Start the server
        log::info!("Starting RPC server on {}", bind_address);
        let server = ServerBuilder::new(io)
            .threads(4)
            .start_http(&bind_address)
            .map_err(|e| WalletError::RPCError(format!("Failed to start RPC server: {}", e)))?;
        
        self.server = Some(server);
        log::info!("RPC server started");
        
        Ok(())
    }
    
    /// Stop the RPC server
    pub fn stop(&mut self) {
        if let Some(server) = self.server.take() {
            log::info!("Stopping RPC server");
            server.close();
            log::info!("RPC server stopped");
        }
    }
}