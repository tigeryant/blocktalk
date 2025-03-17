use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::chain_capnp::chain::Client as ChainClient;
use crate::init_capnp::init::Client as InitClient;
use crate::proxy_capnp::thread::Client as ThreadClient;
use crate::BlockTalkError;

pub struct Connection {
    rpc_handle: JoinHandle<Result<(), capnp::Error>>,
    disconnector: capnp_rpc::Disconnector<twoparty::VatId>,
    thread: ThreadClient,
    chain_client: ChainClient,
}

impl Connection {
    pub async fn connect(socket_path: &str) -> Result<Arc<Self>, BlockTalkError> {
        log::info!("Connecting to Bitcoin node at {}", socket_path);

        let stream = tokio::net::UnixStream::connect(socket_path).await.map_err(|e| {
            log::error!("Failed to connect to Unix socket at {}: {}", socket_path, e);
            BlockTalkError::Io(e)
        })?;
        log::debug!("Unix stream connected successfully");
        let (reader, writer) = stream.into_split();

        log::debug!("Setting up RPC network");
        let network = Box::new(twoparty::VatNetwork::new(
            reader.compat(),
            writer.compat_write(),
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));
        let mut rpc = RpcSystem::new(network, None);
        let init_interface: InitClient = rpc.bootstrap(rpc_twoparty_capnp::Side::Server);
        let disconnector = rpc.get_disconnector();

        log::debug!("Spawning RPC task");
        let rpc_handle = tokio::task::spawn_local(rpc);

        let mk_init_req = init_interface.construct_request();
        let response = mk_init_req.send().promise.await.map_err(|e| {
            log::error!("Failed to initialize connection: {}", e);
            BlockTalkError::Connection(e)
        })?;

        let thread_map = response.get()?.get_thread_map().map_err(|e| {
            log::error!("Failed to get thread map: {}", e);
            BlockTalkError::Connection(e)
        })?;

        let mk_thread_req = thread_map.make_thread_request();
        let response = mk_thread_req.send().promise.await.map_err(|e| {
            log::error!("Failed to create thread: {}", e);
            BlockTalkError::Connection(e)
        })?;

        let thread = response.get()?.get_result().map_err(|e| {
            log::error!("Failed to get thread result: {}", e);
            BlockTalkError::Connection(e)
        })?;
        log::debug!("Thread client established");

        let mut mk_chain_req = init_interface.make_chain_request();
        {
            let mut context = mk_chain_req.get().get_context().map_err(|e| {
                log::error!("Failed to get chain context: {}", e);
                BlockTalkError::Connection(e)
            })?;
            context.set_thread(thread.clone());
        }
        let response = mk_chain_req.send().promise.await.map_err(|e| {
            log::error!("Failed to initialize chain client: {}", e);
            BlockTalkError::Connection(e)
        })?;

        let chain_client = response.get()?.get_result().map_err(|e| {
            log::error!("Failed to get chain client result: {}", e);
            BlockTalkError::Connection(e)
        })?;
        log::debug!("Chain client established");

        log::info!("Connection to node established successfully");
        Ok(Arc::new(Self {
            rpc_handle,
            disconnector,
            thread,
            chain_client,
        }))
    }

    pub async fn disconnect(self) -> Result<(), BlockTalkError> {
        log::info!("Disconnecting from node");
        self.disconnector
            .await
            .map_err(|e| {
                log::error!("Failed to disconnect RPC: {}", e);
                BlockTalkError::Connection(e)
            })?;
        
        match self.rpc_handle.await {
            Ok(result) => {
                result.map_err(|e| {
                    log::error!("RPC handle error during disconnect: {}", e);
                    BlockTalkError::Connection(e)
                })?;
            }
            Err(e) => {
                log::error!("Task join error during disconnect: {}", e);
                return Err(BlockTalkError::node_error(e.to_string(), -1));
            }
        }

        log::info!("Disconnection completed successfully");
        Ok(())
    }

    pub fn chain_client(&self) -> &ChainClient {
        &self.chain_client
    }

    pub fn thread(&self) -> &ThreadClient {
        &self.thread
    }
}
