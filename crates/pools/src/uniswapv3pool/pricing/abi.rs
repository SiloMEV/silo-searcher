use alloy::sol;

sol! {
        /// Interface of the Quoter
        #[derive(Debug, PartialEq, Eq)]
        #[sol(rpc)]
        contract IQuoter {
            function quoteExactInputSingle(address tokenIn, address tokenOut,uint24 fee, uint256 amountIn, uint160 sqrtPriceLimitX96) external returns (uint256 amountOut);
        }
}

sol! {
    #[sol(abi = true, rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface IQuoterV2 {
    function quoteExactInputSingle(QuoteExactInputSingleParams memory params)
        external
        returns (
            uint256 amountOut,
            uint160 sqrtPriceX96After,
            uint32 initializedTicksCrossed,
            uint256 gasEstimate
        );

    struct QuoteExactInputSingleParams {
        address tokenIn;
        address tokenOut;
        uint256 amountIn;
        uint24 fee;
        uint160 sqrtPriceLimitX96;
    }
    }
}

sol! {
    #[sol(abi = true, rpc)]
    #[derive(Debug, PartialEq, Eq)]
    interface ITickLens {
        struct PopulatedTick {
            int24 tick;
            int128 liquidityNet;
            uint128 liquidityGross;
        }

        function getPopulatedTicksInWord(address pool, int16 tickBitmapIndex)
            external
            view
            returns (PopulatedTick[] memory populatedTicks);
    }
}


