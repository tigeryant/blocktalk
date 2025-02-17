use std::sync::Arc;
use tokio::task::{JoinHandle, LocalSet};
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use crate::BlockTalkError;

use crate::init_capnp::init::Client as InitClient;
use crate::chain_capnp::chain::Client as ChainClient;
use crate::proxy_capnp::thread::Client as ThreadClient;
use crate::echo_capnp::echo::Client as EchoClient;

/// Represents a connection to the Bitcoin node
pub struct Connection {
    rpc_handle: JoinHandle<Result<(), capnp::Error>>,
    disconnector: capnp_rpc::Disconnector<twoparty::VatId>,
    thread: ThreadClient,
    chain_client: ChainClient,
}

impl Connection {
    async fn test_echo(init_interface: &InitClient, thread: &ThreadClient) -> Result<(), BlockTalkError> {
        println!("Echo test: Getting echo client...");
        let mut mk_echo_req = init_interface.make_echo_request();
        {
            let mut context = mk_echo_req.get().get_context()?;
            context.set_thread(thread.clone());
        }
        let response = mk_echo_req.send().promise.await?;
        let echo_client: EchoClient = response.get()?.get_result()?;
        
        println!("Echo test: Sending test message...");
        let mut echo_msg_req = echo_client.echo_request();
        {
            let mut params = echo_msg_req.get();
            params.set_echo("test");
        }
        let response = echo_msg_req.send().promise.await?;
        let result = response.get()?.get_result()?;
        
        if result != "test" {
            println!("Echo test: Unexpected response: {:?}", result);
            return Err(BlockTalkError::NodeError("Echo test failed".to_string()));
        }
        println!("Echo test: Success!");
        Ok(())
    }

    /// Create a new connection to the Bitcoin node
    pub async fn connect(socket_path: &str) -> Result<Arc<Self>, BlockTalkError> {
        println!("Connection: Attempting to connect to socket at {}", socket_path);
        
        // Create a LocalSet for running local tasks
        let local = LocalSet::new();
        
        // Run the connection setup in the local set
        let connection = local.run_until(async {
            // Connect to the Unix socket
            println!("Connection: Creating Unix stream connection...");
            let stream = tokio::net::UnixStream::connect(socket_path).await?;
            println!("Connection: Unix stream connected successfully");
            
            let (reader, writer) = stream.into_split();
            println!("Connection: Split stream into reader and writer");
            
            // Set up the RPC network
            println!("Connection: Setting up RPC network...");
            let network = Box::new(twoparty::VatNetwork::new(
                reader.compat(),
                writer.compat_write(),
                rpc_twoparty_capnp::Side::Client,
                Default::default(),
            ));
            println!("Connection: RPC network created");

            // Initialize RPC system
            println!("Connection: Initializing RPC system...");
            let mut rpc = RpcSystem::new(network, None);
            println!("Connection: Getting bootstrap interface...");
            let init_interface: InitClient = rpc.bootstrap(rpc_twoparty_capnp::Side::Server);
            let disconnector = rpc.get_disconnector();
            
            println!("Connection: Spawning RPC task...");
            let rpc_handle = tokio::task::spawn_local(rpc);
            println!("Connection: RPC task spawned");

            // Get thread client
            println!("Connection: Constructing thread request...");
            let mk_init_req = init_interface.construct_request();
            println!("Connection: Sending init request...");
            let response = mk_init_req.send().promise.await?;
            println!("Connection: Got init response");
            
            let thread_map = response.get()?.get_thread_map()?;
            println!("Connection: Got thread map");
            
            let mk_thread_req = thread_map.make_thread_request();
            println!("Connection: Sending thread request...");
            let response = mk_thread_req.send().promise.await?;
            println!("Connection: Got thread response");
            
            let thread = response.get()?.get_result()?;
            println!("Connection: Thread client established");

            // Set up chain client with thread context
            println!("Connection: Setting up chain client...");
            let mut mk_chain_req = init_interface.make_chain_request();
            {
                let mut context = mk_chain_req.get().get_context()?;
                context.set_thread(thread.clone());
            }
            println!("Connection: Sending chain request...");
            let response = mk_chain_req.send().promise.await?;
            println!("Connection: Received chain response");
            
            let chain_client = response.get()?.get_result()?;
            println!("Connection: Got chain client");

            Ok::<_, BlockTalkError>(Arc::new(Self {
                rpc_handle,
                disconnector,
                thread,
                chain_client,
            }))
        }).await?;

        println!("Connection: Setup completed successfully");
        Ok(connection)
    }

    /// Disconnect from the node
    pub async fn disconnect(self) -> Result<(), BlockTalkError> {
        println!("Connection: Starting disconnect...");
        let local = LocalSet::new();
        
        local.run_until(async {
            println!("Connection: Awaiting disconnector...");
            self.disconnector.await.map_err(BlockTalkError::ConnectionError)?;
            println!("Connection: Disconnector completed");
            
            println!("Connection: Awaiting RPC handle...");
            self.rpc_handle.await.map_err(|e| BlockTalkError::NodeError(e.to_string()))?
                .map_err(BlockTalkError::ConnectionError)?;
            println!("Connection: RPC handle completed");
            
            println!("Connection: Disconnect completed successfully");
            Ok(())
        }).await
    }

    /// Get a reference to the chain client
    pub fn chain_client(&self) -> &ChainClient {
        &self.chain_client
    }

    /// Get a reference to the thread client
    pub fn thread(&self) -> &ThreadClient {
        &self.thread
    }
}