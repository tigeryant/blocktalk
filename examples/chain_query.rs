use std::path::Path;
use std::time::Duration;
use tokio::task::LocalSet;

#[tokio::main]
async fn main() -> Result<(), blocktalk::BlockTalkError> {
    // // Initialize BlockTalk by connecting to the Bitcoin node
    // // Typically the socket is in the Bitcoin data directory
    // let socket_path = "../bitcoin/datadir_bdk_wallet/regtest/node.sock";
    // let block_talk = BlockTalk::init(socket_path).await?;
    // println!("Connected");
    // // Get a reference to the chain interface
    // let chain = block_talk.chain();

    // // Get the current tip information
    // let (tip_height, tip_hash) = chain.get_tip().await?;
    // println!("Current tip - Height: {}, Hash: {:?}", tip_height, &tip_hash);

    // Ok(())

    let socket_path = "../bitcoin/datadir_bdk_wallet/regtest/node.sock";

    // Check if socket file exists
    if !Path::new(socket_path).exists() {
        println!("Error: Socket file {} does not exist!", socket_path);
        println!("Please check that:");
        println!("1. Bitcoin node is running");
        println!("2. Bitcoin node is configured to use this Unix socket path");
        println!("3. You have the correct permissions to access the socket");
        return Ok(());
    }

    // Create a LocalSet for running local tasks
    let local = LocalSet::new();

    local
        .run_until(async {
            println!("Initializing BlockTalk...");
            let blocktalk = match tokio::time::timeout(
                Duration::from_secs(5),
                blocktalk::BlockTalk::init(socket_path),
            )
            .await
            {
                Ok(result) => match result {
                    Ok(bt) => bt,
                    Err(e) => {
                        println!("Error initializing BlockTalk: {:?}", e);
                        return Ok(());
                    }
                },
                Err(_) => {
                    println!("Error: Connection timed out after 5 seconds");
                    return Ok(());
                }
            };
            println!("BlockTalk initialized successfully");

            println!("Attempting to get chain tip...");
            match tokio::time::timeout(Duration::from_secs(5), blocktalk.chain().get_tip()).await {
                Ok(result) => match result {
                    Ok((height, hash)) => {
                        println!("Success! Chain tip:");
                        println!("Height: {}", height);
                        println!("Hash: {:?}", hash);
                    }
                    Err(e) => {
                        println!("Error getting chain tip: {:?}", e);
                    }
                },
                Err(_) => {
                    println!("Error: Request timed out after 5 seconds");
                }
            }

            println!("Attempting to get block at height 0...");
            match tokio::time::timeout(
                Duration::from_secs(5),
                blocktalk.chain().get_block_at_height(0),
            )
            .await
            {
                Ok(result) => match result {
                    Ok(block) => {
                        println!("Success! Block at height 0: {:?}", block);
                    }
                    Err(e) => {
                        println!("Error getting block at height 0 {:?}", e);
                    }
                },
                Err(_) => {
                    println!("Error: Request timed out after 5 seconds");
                }
            }

            // println!("Disconnecting...");
            // blocktalk.disconnect().await?;
            // println!("Disconnected successfully");

            Ok(())
        })
        .await
}
