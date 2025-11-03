-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Pools table: Uniswap v4 pool information
CREATE TABLE pools (
    pool_id VARCHAR(66) PRIMARY KEY,  -- Pool identifier (address or hash)
    token0 VARCHAR(42) NOT NULL,      -- Address of token0
    token1 VARCHAR(42) NOT NULL,      -- Address of token1
    fee_tier INTEGER NOT NULL,        -- Fee tier (uint24)
    tick_spacing INTEGER NOT NULL,    -- Tick spacing (int24)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Positions table: LP position NFTs
CREATE TABLE positions (
    id BIGSERIAL PRIMARY KEY,
    nft_id VARCHAR(78) NOT NULL UNIQUE,  -- NFT token ID (uint256 as string)
    owner VARCHAR(42) NOT NULL,           -- Owner address
    pool_id VARCHAR(66) NOT NULL REFERENCES pools(pool_id) ON DELETE CASCADE,
    tick_lower INTEGER NOT NULL,          -- Lower tick boundary (int24)
    tick_upper INTEGER NOT NULL,          -- Upper tick boundary (int24)
    liquidity NUMERIC(78, 0) NOT NULL,    -- Liquidity amount (uint128)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Swaps table: Swap events for fee calculations
CREATE TABLE swaps (
    id BIGSERIAL PRIMARY KEY,
    tx_hash VARCHAR(66) NOT NULL,         -- Transaction hash
    pool_id VARCHAR(66) NOT NULL REFERENCES pools(pool_id) ON DELETE CASCADE,
    amount0 NUMERIC(78, 0) NOT NULL,      -- Token0 amount (int256, can be negative)
    amount1 NUMERIC(78, 0) NOT NULL,      -- Token1 amount (int256, can be negative)
    timestamp TIMESTAMPTZ NOT NULL,
    UNIQUE(tx_hash, pool_id)              -- Prevent duplicate swap records
);

-- Position snapshots table: Time-series data for P&L tracking
CREATE TABLE position_snapshots (
    id BIGSERIAL,
    position_id BIGINT NOT NULL REFERENCES positions(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL,
    fees_earned NUMERIC(78, 18) NOT NULL,  -- Fees earned (decimal with 18 precision)
    liquidity NUMERIC(78, 0) NOT NULL,     -- Current liquidity
    price NUMERIC(78, 18) NOT NULL,        -- Current price (decimal)
    PRIMARY KEY (id, timestamp)
);

-- Convert position_snapshots to TimescaleDB hypertable
-- This enables efficient time-series queries and automatic partitioning
SELECT create_hypertable('position_snapshots', 'timestamp');

-- Indexes for efficient queries
CREATE INDEX idx_positions_owner ON positions(owner);
CREATE INDEX idx_positions_pool_id ON positions(pool_id);
CREATE INDEX idx_positions_created_at ON positions(created_at);

CREATE INDEX idx_swaps_pool_id ON swaps(pool_id);
CREATE INDEX idx_swaps_timestamp ON swaps(timestamp);
CREATE INDEX idx_swaps_tx_hash ON swaps(tx_hash);

CREATE INDEX idx_position_snapshots_position_id ON position_snapshots(position_id, timestamp DESC);

-- Create a composite index for efficient position lookups by owner and pool
CREATE INDEX idx_positions_owner_pool ON positions(owner, pool_id);
