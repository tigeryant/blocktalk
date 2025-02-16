use blocktalk::BlockTalk;

#[tokio::main]
async fn main() -> Result<(), blocktalk::BlockTalkError> {
    // Initialize BlockTalk by connecting to the Bitcoin node
    // Typically the socket is in the Bitcoin data directory
    let socket_path = "../bitcoin/datadir_bdk_wallet/regtest/node.sock";
    let block_talk = BlockTalk::init(socket_path).await?;
    
    // Get a reference to the chain interface
    let chain = block_talk.chain();

    // Get the current tip information
    let (tip_height, tip_hash) = chain.get_tip().await?;
    println!("Current tip - Height: {}, Hash: {:?}", tip_height, &tip_hash);

    Ok(())
}