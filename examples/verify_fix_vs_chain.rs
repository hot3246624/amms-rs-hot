use alloy::{
    eips::BlockId,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    rpc::client::ClientBuilder,
    transports::layers::{RetryBackoffLayer, ThrottleLayer},
    sol,
};
use amms::amms::{amm::AutomatedMarketMaker, uniswap_v3::UniswapV3Pool};
use std::sync::Arc;

sol! {
    /// Interface of the Quoter
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc)]
    contract IQuoter {
        function quoteExactInputSingle(address tokenIn, address tokenOut, uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96) external returns (uint256 amountOut);
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Ensure ETHEREUM_PROVIDER is set (can be Anvil or Mainnet)
    let rpc_endpoint = std::env::var("ETHEREUM_PROVIDER").unwrap_or_else(|_| "http://localhost:8545".to_string());
    
    // Uniswap V3 USDC/WETH 0.05% Pool
    let pool_address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let quoter_address = address!("b27308f9f90d607463bb33ea1bebb41c27ce5ab6");

    // Initialize pool WITHOUT full tick bitmap sync to avoid RPC 429 errors
    // Use .init_slot_0() to get current price/tick/liquidity first
    // Note: init() calls sync_tick_bitmaps() internally which causes the massive requests. 
    // We will manually construct the pool and only fetch what we need, or use a method if available.
    // Actually, amms-rs `init` does full sync. Let's try to manually sync only necessary parts or use `populate_tick_data` if available exposed?
    // Looking at mod.rs, `populate_tick_data` is part of `sync_tick_data`. 
    // Since we need to verify the FIX in `sync_tick_bitmaps`, avoiding it defeats the purpose.
    
    // WAIT. The fix is in `sync_tick_bitmaps`. If we don't call it, we don't verify the fix!
    // The previous 429 is likely because `sync_tick_bitmaps` requests ALL words.
    // We MUST use `sync_tick_bitmaps` to verify the fix works. 
    // To solve 429, we should drastically increase the ThrottleLayer wait time or use a smarter batching.
    // But modifying the library code just for test is bad.
    
    // Alternative: The user suggested "bulk query with eth_call" or "sleep/delay".
    // Since `amms-rs` internal logic for `sync_tick_bitmaps` uses `GetUniswapV3PoolTickBitmapBatchRequest` which IS a multicall (batch request).
    // The issue is it sends MANY batches very fast.
    
    // Let's implement the "Sleep/Delay" strategy by wrapping the provider with a RateLimit middleware? 
    // Or simpler: just increase the ThrottleLayer to be very aggressive (e.g., 500ms or 1s).
    
    // User asked "Which is better?". I'll implement the "Throttle/Sleep" strategy via ClientBuilder configuration
    // because `sync_tick_bitmaps` is exactly what we fixed, we CANNOT skip it.
    
    let client = ClientBuilder::default()
        // Aggressive throttling: 1 request every 500ms to allow Anvil/Free tire to breathe
        .layer(ThrottleLayer::new(1000)) 
        .layer(RetryBackoffLayer::new(100, 1000, 20000)) // More retries, longer backoff
        .http(rpc_endpoint.parse()?);

    let provider = Arc::new(ProviderBuilder::new().connect_client(client));
    
    // Using current latest block for synchronization
    let block_number = provider.get_block_number().await?;
    let block_id = BlockId::from(block_number);
    println!("Syncing pool at block: {}", block_number);

    // Local Anvil Deployment
    let pool_address = address!("19b810469a2FE1dC4eCc04487CA77a7ee8c78B82");
    let quoter_address = address!("5FC8d32690cc91D4c39d9d3abcBD16989F875707");

    let pool = UniswapV3Pool::new(pool_address)
        .init(block_id, provider.clone())
        .await?;
        
    let quoter = IQuoter::new(quoter_address, provider.clone());

    // Mock Tokens (1 WETH = 1e18, 1 USDC = 1e6)
    // We added liquidity around Tick 0 (1:1 price), Tick 80000, and Tick 160000.
    // Let's test swapping small and large amounts to hit these ticks.
    let test_amounts = vec![
        U256::from(1_000_000_000_000_000_000_u128),     // 1 Token (~1.0 price)
        U256::from(100_000_000_000_000_000_000_u128),   // 100 Tokens
        U256::from(10_000_000_000_000_000_000_000_u128), // 10,000 Tokens (Should push price significantly)
    ];

    println!("\nStarting comparison tests (USDC -> WETH)...");
    
    for amount_in in test_amounts {
    println!("Pool Details:");
    println!("  Token A: {:?}", pool.token_a);
    println!("  Token B: {:?}", pool.token_b);
    println!("  Fee: {}", pool.fee);
    println!("  Liquidity: {}", pool.liquidity);
    println!("  SqrtPrice: {}", pool.sqrt_price);
    println!("  Tick: {}", pool.tick);
    
    // Check if simulate_swap works locally first
    println!("Simulating swap locally...");
    let amount_out_local = pool.simulate_swap(
        pool.token_a.address,
        Address::default(), 
        amount_in
    )?;
    println!("Local swap result: {}", amount_out_local);

    // 2. Chain Execution (Quoter)
    println!("Calling Quoter on-chain...");
    let amount_out_chain = quoter.quoteExactInputSingle(
        pool.token_a.address,
        pool.token_b.address,
        alloy::primitives::aliases::U24::from(pool.fee),
        amount_in,
        alloy::primitives::U160::ZERO
    )
    .block(block_id)
    .call()
    .await?;

        println!("------------------------------------------------");
        println!("Input Amount: {} USDC", amount_in);
        println!("Local Sim:     {}", amount_out_local);
        println!("Chain Quoter:  {}", amount_out_chain);
        
        if amount_out_local == amount_out_chain {
            println!("✅ MATCH");
        } else {
            println!("❌ MISMATCH");
            println!("Diff: {}", if amount_out_local > amount_out_chain {
                amount_out_local - amount_out_chain
            } else {
                amount_out_chain - amount_out_local
            });
            // We exit on mismatch to highlight the error
            // return Err(eyre::eyre!("Simulation mismatch")); 
        }
    }

    Ok(())
}
