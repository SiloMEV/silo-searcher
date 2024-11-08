use alloy::sol;

sol! {
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc)]
    contract IUniswapV3Pool {
        event Swap(address indexed sender, address indexed recipient, int256 amount0, int256 amount1, uint160 sqrtPriceX96, uint128 liquidity, int24 tick);
        event Burn(address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1);
        event Mint(address sender, address indexed owner, int24 indexed tickLower, int24 indexed tickUpper, uint128 amount, uint256 amount0, uint256 amount1);
        function token0() external view returns (address);
        function token1() external view returns (address);
        function liquidity() external view returns (uint128);
        function factory() external view returns (address);
        function slot0() external view returns (
                uint160 sqrtPriceX96,
                int24 tick,
                uint16 observationIndex,
                uint16 observationCardinality,
                uint16 observationCardinalityNext,
                uint8 feeProtocol,
                bool unlocked
        );
        function fee() external view returns (uint24);
        function tickSpacing() external view returns (int24);
        function ticks(int24 tick) external view returns (uint128, int128, uint256, uint256, int56, uint160, uint32, bool);
        function tickBitmap(int16 wordPosition) external view returns (uint256);
        function swap(address recipient, bool zeroForOne, int256 amountSpecified, uint160 sqrtPriceLimitX96, bytes calldata data) external returns (int256, int256);
    }
}
