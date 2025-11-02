# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Stillwater is a Rust web service built with Axum, PostgreSQL, and Redis. The project uses Rust 2024 edition and follows an async-first architecture with Tokio runtime.

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

### Application Structure

The application follows a layered architecture:

1. **Entry Point** (`src/main.rs`): Initializes tracing, database pool with migrations, Redis client, blockchain service, and Axum server
2. **Configuration** (`src/config.rs`): Initialization functions for logging, database, Redis, and blockchain
3. **State Management** (`src/state.rs`): AppState struct that holds shared db_pool, redis_client, and blockchain service, cloned across handlers
4. **Services Layer** (`src/services/`): Database, cache, and blockchain operations
5. **Handlers** (`src/handlers/`): Axum route handlers (currently stubbed)
6. **Contracts** (`src/contracts/`): Uniswap v4 contract bindings and types

### State Sharing Pattern

The application uses Axum's state extraction pattern:
- `AppState` is created once at startup with db_pool and redis_client
- Passed to Router via `.with_state(app_state)`
- Handlers extract state using `State(state): State<AppState>`
- AppState derives Clone for efficient sharing across handlers

### Database & Caching

- **PostgreSQL**: Connection pool managed by SQLx (max 5 connections)
- **Redis**: Client initialized at startup, shared via AppState
- **Migrations**: SQLx migrations in `migrations/` directory, run automatically on startup
- **Environment**: Requires DATABASE_URL and REDIS_URL in .env file

### Dependencies

Key dependencies and their use cases:
- `axum 0.8`: Web framework
- `tokio`: Async runtime with "full" features
- `sqlx 0.8`: Postgres database with async support and migrations
- `redis 0.27`: Redis client with tokio compatibility
- `alloy 0.8`: Ethereum library for smart contract interactions
- `alloy-sol-types 0.8`: Solidity type bindings for Rust
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

# Ethereum RPC Configuration
ETHEREUM_RPC_URL=http://localhost:8545  # Local Anvil
# For production use: https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
```

## Development Workflow

1. Start Docker services: `cd docker && just dstart`
2. Start local Ethereum node (optional, for local testing): `anvil`
3. Migrations run automatically on `cargo run`
4. Server starts on `http://127.0.0.1:3000`
5. Add new routes by extending Router in `src/main.rs`
6. Implement service functions in `src/services/database.rs`, `src/services/cache.rs`, or `src/services/blockchain.rs`
7. Add handler functions and wire them to routes

## Uniswap v4 Integration

### Smart Contracts

Uniswap v4 core contracts are located in `contracts/lib/v4-core/`. The project uses Foundry for contract management.

### Contract Bindings

Rust bindings for Uniswap v4 contracts are auto-generated using Alloy's `sol!` macro in `src/contracts/mod.rs`. Key contracts included:

- **IPoolManager**: Main pool manager interface with functions for initializing pools, adding/removing liquidity, and swapping
- **IHooks**: Hook interface for implementing custom pool behaviors

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
3. Add bindings to `src/contracts/mod.rs` if needed
4. Interact with hooks via the blockchain service in `src/services/blockchain.rs`

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

The `BlockchainService` in `src/services/blockchain.rs` provides:
- Ethereum RPC provider (via Alloy)
- Contract interaction helpers
- Access via `state.blockchain` in handlers
