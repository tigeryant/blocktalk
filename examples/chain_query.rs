// use blocktalk::BlockTalk;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let blocktalk = BlockTalk::connect("/path/to/socket").await?;
//     let chain = blocktalk.chain();
    
//     // Get current tip
//     let tip = chain.get_tip().await?;
//     println!("Current tip: height={}, hash={}", tip.height, tip.hash);
    
//     // Get last 10 blocks
//     for height in (tip.height - 10)..=tip.height {
//         let block = chain.get_block(height).await?;
//         println!("Block {} has {} transactions", 
//             block.block_hash(),
//             block.txdata.len()
//         );
//     }
    
//     Ok(())
// }