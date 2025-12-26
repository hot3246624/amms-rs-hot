// SPDX-License-Identifier: MIT
pragma solidity ^0.7.6;
import "forge-std/Script.sol";
import "@uniswap/v3-core/contracts/UniswapV3Pool.sol";

contract GetInitHash is Script {
    function run() external view {
        bytes32 initCodeHash = keccak256(type(UniswapV3Pool).creationCode);
        console.logBytes32(initCodeHash);
    }
}
