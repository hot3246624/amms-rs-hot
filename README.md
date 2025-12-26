# amms-rs [![Github Actions][gha-badge]][gha] [![Chat][tg-badge]][tg-url]

[gha]: https://github.com/darkforestry/amms-rs/actions
[gha-badge]: https://github.com/darkforestry/amms-rs/actions/workflows/ci.yml/badge.svg
[tg-url]: https://t.me/amms_rs
[tg-badge]: https://img.shields.io/badge/chat-telegram-blue

**amms-rs** is a powerful Rust library designed to interact with Automated Market Makers (AMMs) across EVM-compatible chains. It leverages **Alloy** for efficient and robust blockchain interactions.

**amms-rs** 是一个强大的 Rust 库，用于与 EVM 兼容链上的自动化做市商 (AMM) 进行交互。它基于 **Alloy** 构建，以提供高效且稳健的区块链交互能力。

---

## 🇺🇸 English Usage Guide

### Key Features
- **Discovery**: Automatically find factory contracts and pools.
- **Syncing**: Keep pool states (liquidity, ticks, reserves) synchronized with the blockchain.
- **Simulation**: Precisely simulate swaps (including Uniswap V3 tick logic) locally.
- **State Space**: Manage a local cache of blockchain state to handle reorgs and updates efficiently.

### Supported AMMs

| AMM             | Status |
| --------------- | ------ |
| UniswapV2       | ✅     |
| UniswapV3       | ✅     |
| Balancer        | ✅     |
| ERC4626 Vaults  | ✅     |

### Installation

Add `amms` to your `Cargo.toml`. Since this library uses `Alloy`, ensure you have compatible versions.

```toml
[dependencies]
amms = { git = "https://github.com/darkforestry/amms-rs" } # Or specific version
alloy = { version = "0.1", features = ["full"] }
tokio = { version = "1", features = ["full"] }
```

### Example: Simulate a Uniswap V3 Swap

Here is a simple example of how to initialize a Uniswap V3 pool and simulate a swap.

```rust
use alloy::eips::BlockId;
use alloy::primitives::{address, Address, U256};
use alloy::providers::ProviderBuilder;
use amms::amms::amm::AutomatedMarketMaker;
use amms::amms::uniswap_v3::UniswapV3Pool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // 1. Setup Provider (RPC URL from environment variable)
    let rpc_endpoint = std::env::var("ETHEREUM_PROVIDER")?;
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint.parse()?));

    // 2. Initialize the Pool (e.g., USDC/WETH on Ethereum)
    // The library will automatically fetch metadata, current state, and tick data.
    let pool_address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let pool = UniswapV3Pool::new(pool_address)
        .init(BlockId::latest(), provider)
        .await?;

    // 3. Simulate Swap
    // Simulate selling 1,000,000 units of Token A (e.g., 1 USDC)
    let amount_in = U256::from(1000000);
    let amount_out = pool.simulate_swap(
        pool.token_a.address, // Token In
        Address::default(),   // Token Out (not needed for 2-token pools)
        amount_in,
    )?;

    println!("Simulated Amount Out: {:?}", amount_out);

    Ok(())
}
```

---

## 🇨🇳 中文使用说明

### 主要功能
- **发现 (Discovery)**: 自动查找工厂合约和交易池。
- **同步 (Syncing)**: 保持池状态（流动性、Tick、储备金）与区块链同步。
- **模拟 (Simulation)**: 在本地精确模拟交易（包括 Uniswap V3 的复杂 Tick 逻辑）。
- **状态空间 (State Space)**: 管理本地区块链状态缓存，有效处理重组 (Reorgs) 和更新。

### 支持的 AMM 协议

| 协议             | 状态 |
| --------------- | ------ |
| UniswapV2       | ✅     |
| UniswapV3       | ✅     |
| Balancer        | ✅     |
| ERC4626 Vaults  | ✅     |

### 安装

将 `amms` 添加到你的 `Cargo.toml` 中。由于本项目基于 `Alloy`，请确保版本兼容。

```toml
[dependencies]
amms = { git = "https://github.com/darkforestry/amms-rs" } # 或者指定版本
alloy = { version = "0.1", features = ["full"] }
tokio = { version = "1", features = ["full"] }
```

### 示例：模拟 Uniswap V3 交易

以下是一个简单的示例，展示如何初始化 Uniswap V3 池并模拟交易。

```rust
use alloy::eips::BlockId;
use alloy::primitives::{address, Address, U256};
use alloy::providers::ProviderBuilder;
use amms::amms::amm::AutomatedMarketMaker;
use amms::amms::uniswap_v3::UniswapV3Pool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // 1. 设置 Provider (从环境变量获取 RPC URL)
    let rpc_endpoint = std::env::var("ETHEREUM_PROVIDER")?;
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint.parse()?));

    // 2. 初始化池子 (例如 Ethereum 上的 USDC/WETH)
    // 库会自动获取元数据、当前状态和 Tick bitmap 数据。
    let pool_address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
    let pool = UniswapV3Pool::new(pool_address)
        .init(BlockId::latest(), provider)
        .await?;

    // 3. 模拟交易
    // 模拟卖出 1,000,000 单位的 Token A (例如 1 USDC)
    let amount_in = U256::from(1000000);
    let amount_out = pool.simulate_swap(
        pool.token_a.address, // 输入代币
        Address::default(),   // 输出代币 (对于只有两个代币的池子，此处可忽略)
        amount_in,
    )?;

    println!("模拟输出金额: {:?}", amount_out);

    Ok(())
}
```
