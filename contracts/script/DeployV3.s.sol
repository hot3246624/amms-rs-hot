// SPDX-License-Identifier: MIT
pragma solidity ^0.7.6;
pragma abicoder v2;

import "forge-std/Script.sol";
import "@uniswap/v3-core/contracts/UniswapV3Factory.sol";
import "@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol";
import "@uniswap/v3-periphery/contracts/lens/Quoter.sol";

// Minimal Mock ERC20
contract MockERC20 {
    string public name;
    string public symbol;
    uint8 public decimals;
    uint256 public totalSupply;
    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    constructor(string memory _name, string memory _symbol, uint8 _decimals) {
        name = _name;
        symbol = _symbol;
        decimals = _decimals;
    }

    function mint(address to, uint256 amount) public {
        balanceOf[to] += amount;
        totalSupply += amount;
    }

    function approve(address spender, uint256 amount) public returns (bool) {
        allowance[msg.sender][spender] = amount;
        return true;
    }

    function transfer(address to, uint256 amount) public returns (bool) {
        return transferFrom(msg.sender, to, amount);
    }

    function transferFrom(address from, address to, uint256 amount) public returns (bool) {
        require(balanceOf[from] >= amount, "Insufficient balance");
        if (from != msg.sender && allowance[from][msg.sender] != type(uint256).max) {
             require(allowance[from][msg.sender] >= amount, "Insufficient allowance");
             allowance[from][msg.sender] -= amount;
        }
        balanceOf[from] -= amount;
        balanceOf[to] += amount;
        return true;
    }
}

// Interface to mint on pool directly
interface IUniswapV3PoolMint {
    function mint(
        address recipient,
        int24 tickLower,
        int24 tickUpper,
        uint128 amount,
        bytes calldata data
    ) external returns (uint256 amount0, uint256 amount1);
    function initialize(uint160 sqrtPriceX96) external;
}

contract DeployV3 is Script {
    MockERC20 public token0; 
    MockERC20 public token1; 
    
    UniswapV3Factory public factory;
    Quoter public quoter;

    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy Tokens
        MockERC20 tA = new MockERC20("Wrapped Ether", "WETH", 18);
        MockERC20 tB = new MockERC20("USD Coin", "USDC", 6);

        if (address(tA) < address(tB)) {
            token0 = tA;
            token1 = tB;
        } else {
            token0 = tB;
            token1 = tA;
        }

        // Mint supply
        token0.mint(deployer, 1_000_000_000 * 10**18);
        token1.mint(deployer, 1_000_000_000 * 10**18);

        // 2. Deploy Factory
        factory = new UniswapV3Factory();
        
        // 3. Deploy Quoter (Only Periphery contract needed)
        quoter = new Quoter(address(factory), address(token0));

        // 4. Create Pool (Fee 1% = 10000, TickSpacing 200)
        address poolAddress = factory.createPool(address(token0), address(token1), 10000);
        IUniswapV3PoolMint pool = IUniswapV3PoolMint(poolAddress);

        // 5. Initialize Pool
        // sqrt(1) * 2^96
        uint160 sqrtPriceX96 = 79228162514264337593543950336;
        pool.initialize(sqrtPriceX96);

        // 6. Deploy Liquidity Provider Helper
        LiquidityProvider lp = new LiquidityProvider();
        token0.approve(address(lp), type(uint256).max);
        token1.approve(address(lp), type(uint256).max);
        
        // Transfer tokens to LP for adding liquidity
        token0.transfer(address(lp), 1_000_000 * 10**18);
        token1.transfer(address(lp), 1_000_000 * 10**18);

        // Position 1: Around current tick (0)
        lp.addLiquidity(poolAddress, address(token0), address(token1), -2000, 2000, 1000 * 10**18, deployer);

        // Position 2: Way up (Word ~312)
        lp.addLiquidity(poolAddress, address(token0), address(token1), 80000, 82000, 1000 * 10**18, deployer);

        // Position 3: Even Higher
        lp.addLiquidity(poolAddress, address(token0), address(token1), 160000, 162000, 1000 * 10**18, deployer);

        console.log("Deployed Environments:");
        console.log("Token0:", address(token0));
        console.log("Token1:", address(token1));
        console.log("Factory:", address(factory));
        console.log("Pool:", poolAddress);
        console.log("Quoter:", address(quoter));

        vm.stopBroadcast();
    }
}

contract LiquidityProvider {
    function addLiquidity(
        address poolAddress, 
        address t0, 
        address t1, 
        int24 tickLower, 
        int24 tickUpper, 
        uint128 amount,
        address deployer
    ) external {
        IUniswapV3PoolMint(poolAddress).mint(
            deployer,
            tickLower,
            tickUpper,
            amount,
            abi.encode(t0, t1)
        );
    }

    function uniswapV3MintCallback(
        uint256 amount0Owed,
        uint256 amount1Owed,
        bytes calldata data
    ) external {
        (address t0, address t1) = abi.decode(data, (address, address));
        if (amount0Owed > 0) MockERC20(t0).transfer(msg.sender, amount0Owed);
        if (amount1Owed > 0) MockERC20(t1).transfer(msg.sender, amount1Owed);
    }
}
