# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stillwater is a professional liquidity management platform for Uniswap v4 LPs on Unichain Sepolia. The project is built with Rust using Axum, PostgreSQL with TimescaleDB, and Redis. It uses Rust 2024 edition and follows an async-first architecture with Tokio runtime.

### Core Value
Accurate P&L tracking (fees - IL - gas) with proactive warnings for liquidity positions.

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Run the application (requires Docker services)
cargo run

# Run in release mode
cargo build --release
cargo run --release

# Check code without building
cargo check

# Format code (follows rustfmt.toml: 100 char width, 4 spaces, Unix line endings)
cargo fmt

# Run linter
cargo clippy
```

### Testing
```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
```

### Database Management
```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations (also runs automatically on app startup)
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Docker Services
```bash
# Start PostgreSQL and Redis containers
cd docker && just dstart

# Stop containers
cd docker && just dkill

# Or using docker-compose directly
docker-compose --env-file .env -f docker/docker-compose.yml up -d
```

## Architecture

### Workspace Structure

The project is organized as a Cargo workspace with 5 crates:

1. **`crates/models`** - Shared domain types, blockchain service, and Uniswap v4 contract bindings
2. **`crates/db`** - Database layer with sqlx for position, swap, and pool data
3. **`crates/indexer`** - The Graph client for fetching Uniswap v4 data
4. **`crates/analytics`** - P&L, IL (impermanent loss), and health calculations
5. **`crates/api`** - REST API with Axum (main binary)

### Application Structure

The application follows a modular monolith architecture:

1. **Entry Point** (`crates/api/src/main.rs`): Initializes tracing, database pool with migrations, Redis client, blockchain service, and Axum server
2. **Configuration** (`crates/api/src/config.rs`): Initialization functions for logging, database, Redis, and blockchain
3. **State Management** (`crates/api/src/state.rs`): AppState struct that holds shared db_pool, redis_client, and blockchain service, cloned across handlers
4. **Models** (`crates/models/`): Blockchain service and Uniswap v4 contract bindings
5. **Database** (`crates/db/`): Database operations (to be implemented)
6. **Indexer** (`crates/indexer/`): The Graph integration (to be implemented)
7. **Analytics** (`crates/analytics/`): P&L calculations (to be implemented)

### State Sharing Pattern

The application uses Axum's state extraction pattern:
- `AppState` is created once at startup with db_pool and redis_client
- Passed to Router via `.with_state(app_state)`
- Handlers extract state using `State(state): State<AppState>`
- AppState derives Clone for efficient sharing across handlers

### Database & Caching

- **PostgreSQL with TimescaleDB**: Connection pool managed by SQLx (max 5 connections); TimescaleDB extension for time-series data
- **Redis**: Client initialized at startup, shared via AppState
- **Migrations**: SQLx migrations in `migrations/` directory, run automatically on startup
- **Environment**: Requires DATABASE_URL, REDIS_URL, ETHEREUM_RPC_URL, and GRAPH_API_URL in .env file

### Dependencies

Key dependencies and their use cases:
- `axum 0.8`: Web framework
- `tokio`: Async runtime with "full" features
- `sqlx 0.8`: Postgres database with async support and migrations
- `redis 0.27`: Redis client with tokio compatibility
- `reqwest 0.12`: HTTP client for The Graph API calls
- `alloy 0.8`: Ethereum library for smart contract interactions
- `alloy-sol-types 0.8`: Solidity type bindings for Rust
- `rust_decimal`: Precise decimal math for P&L calculations
- `chrono`: DateTime handling for time-series data
- `uuid`: Unique identifiers for positions
- `serde/serde_json`: JSON serialization
- `tracing/tracing-subscriber`: Logging (default level: info)
- `dotenv`: Environment variable loading
- `anyhow`: Error handling

## Environment Setup

Create a `.env` file in the project root:
```
POSTGRES_USER=admin
POSTGRES_PASSWORD=stillwater
POSTGRES_DB=stillwater
DATABASE_URL=postgres://admin:stillwater@localhost:5432/stillwater
REDIS_URL=redis://localhost:6379

# Ethereum RPC Configuration (Unichain Sepolia)
ETHEREUM_RPC_URL=https://unichain-sepolia.g.alchemy.com/v2/YOUR_API_KEY

# The Graph API Configuration (Uniswap v4 on Unichain Sepolia)
GRAPH_API_URL=https://gateway.thegraph.com/api/YOUR_API_KEY/subgraphs/id/GWdEPuFDzBVc2EDC4grZkN5zecqKPYYP2okXd39fnE5R

# For local testing with Anvil:
# ETHEREUM_RPC_URL=http://localhost:8545
```

## Development Workflow

1. Start Docker services: `cd docker && just dstart`
2. Migrations run automatically when running the API: `cargo run -p stillwater-api`
3. Server starts on `http://127.0.0.1:3000`
4. Add new routes by extending Router in `crates/api/src/main.rs`
5. Implement database functions in `crates/db/src/lib.rs`
6. Implement analytics functions in `crates/analytics/src/lib.rs`
7. Implement indexer functions in `crates/indexer/src/lib.rs`
8. Add handler functions and wire them to routes

## Uniswap v4 Integration

### Smart Contracts

Uniswap v4 core contracts are located in `contracts/lib/v4-core/`. The project uses Foundry for contract management.

### Contract Bindings

Rust bindings for Uniswap v4 contracts are auto-generated using Alloy's `sol!` macro in `crates/models/src/contracts/mod.rs`. Key contracts included:

- **IPoolManager**: Main pool manager interface with functions for initializing pools, adding/removing liquidity, and swapping
- **IHooks**: Hook interface for implementing custom pool behaviors
- **IERC20Minimal**: ERC20 token interface
- **IERC6909Claims**: ERC6909 claims interface
- **IUnlockCallback**: Unlock callback interface for PoolManager interactions

### Hooks

Uniswap v4 hooks allow custom logic to be executed at various points in the pool lifecycle:

- `beforeInitialize` / `afterInitialize`: Pool initialization
- `beforeSwap` / `afterSwap`: Token swaps
- `beforeAddLiquidity` / `afterAddLiquidity`: Liquidity provision
- `beforeRemoveLiquidity` / `afterRemoveLiquidity`: Liquidity removal
- `beforeDonate` / `afterDonate`: Donations

To create custom hooks:
1. Create a new Solidity contract in `contracts/src/` that implements `IHooks`
2. Compile with `forge build --root contracts`
3. Add bindings to `crates/models/src/contracts/mod.rs` if needed
4. Interact with hooks via the blockchain service in `crates/models/src/blockchain.rs`

### Local Testing

Start a local Anvil node for testing:
```bash
anvil
```

Deploy contracts to local network:
```bash
cd contracts
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

### Blockchain Service

The `BlockchainService` in `crates/models/src/blockchain.rs` provides:
- Ethereum RPC provider (via Alloy) connected to Unichain Sepolia
- Contract interaction helpers
- Access via `state.blockchain` in handlers

### The Graph Integration

The indexer crate (`crates/indexer/`) provides integration with The Graph for querying Uniswap v4 data:
- Position data (NFT IDs, owners, tick ranges, liquidity)
- Swap events for fee calculations
- Pool information
- Polling mechanism for real-time updates
