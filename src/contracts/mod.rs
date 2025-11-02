use alloy::sol;

// Core Uniswap v4 types and interfaces
// These are inline Solidity definitions for the key contracts

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IPoolManager {
        struct PoolKey {
            address currency0;
            address currency1;
            uint24 fee;
            int24 tickSpacing;
            address hooks;
        }

        struct ModifyLiquidityParams {
            int24 tickLower;
            int24 tickUpper;
            int256 liquidityDelta;
            bytes32 salt;
        }

        struct SwapParams {
            bool zeroForOne;
            int256 amountSpecified;
            uint160 sqrtPriceLimitX96;
        }

        function initialize(PoolKey memory key, uint160 sqrtPriceX96) external returns (int24 tick);
        function modifyLiquidity(PoolKey memory key, ModifyLiquidityParams memory params, bytes calldata hookData) external returns (int256, int256);
        function swap(PoolKey memory key, SwapParams memory params, bytes calldata hookData) external returns (int256, int256);
        function updateDynamicLPFee(PoolKey memory key, uint24 newDynamicLPFee) external;
    }
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IHooks {
        struct Permissions {
            bool beforeInitialize;
            bool afterInitialize;
            bool beforeAddLiquidity;
            bool afterAddLiquidity;
            bool beforeRemoveLiquidity;
            bool afterRemoveLiquidity;
            bool beforeSwap;
            bool afterSwap;
            bool beforeDonate;
            bool afterDonate;
            bool beforeSwapReturnDelta;
            bool afterSwapReturnDelta;
            bool afterAddLiquidityReturnDelta;
            bool afterRemoveLiquidityReturnDelta;
        }

        function getHookPermissions() external view returns (Permissions memory);
    }
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IUnlockCallback {
        /// @notice Called by the pool manager when the contract unlocks the pool manager
        /// @dev Critical: All PoolManager operations (swap, modifyLiquidity, etc.) must be called within unlockCallback
        function unlockCallback(bytes calldata data) external returns (bytes memory);
    }
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IERC20Minimal {
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address from, address to, uint256 amount) external returns (bool);
    }
}

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    interface IERC6909Claims {
        function balanceOf(address owner, uint256 id) external view returns (uint256);
        function transfer(address receiver, uint256 id, uint256 amount) external returns (bool);
        function transferFrom(address sender, address receiver, uint256 id, uint256 amount) external returns (bool);
    }
}

// Re-export the generated types
pub use IPoolManager::*;
pub use IHooks::*;
pub use IUnlockCallback::*;
pub use IERC20Minimal::*;
pub use IERC6909Claims::*;
