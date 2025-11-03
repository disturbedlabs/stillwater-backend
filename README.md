# Stillwater

A Uniswap v4 liquidity position tracking and analytics platform for Unichain Sepolia. Built with Rust, Axum, PostgreSQL (TimescaleDB), and The Graph.

## Features

- **Real-time Position Tracking** - Monitor Uniswap v4 LP positions on Unichain Sepolia
- **P&L Analytics** - Calculate fees earned, impermanent loss, and net P&L
- **Health Monitoring** - Track position health status (Healthy/Warning/Critical)
- **Time-Series Storage** - TimescaleDB for efficient historical data queries
- **The Graph Integration** - Sync blockchain data via GraphQL

## Tech Stack

- **Rust 2024 Edition** - Modern, safe systems programming with Cargo workspace
- **Axum 0.8** - Ergonomic web framework built on Tokio
- **PostgreSQL 17 + TimescaleDB** - Time-series optimized database with SQLx
- **The Graph** - Decentralized blockchain indexing via GraphQL
- **Alloy** - Ethereum library for smart contract interactions
- **Redis** - In-memory caching layer
- **Docker** - Containerized development environment

## Architecture

Stillwater is organized as a Cargo workspace with 5 crates:

1. **stillwater-models** (`crates/models/`) - Domain types and contract bindings
   - Pool, Position, Swap, PositionPnL models
   - Uniswap v4 contract bindings (Alloy)
   - Blockchain service for RPC interactions

2. **stillwater-db** (`crates/db/`) - Database operations
   - CRUD operations for all entities
   - TimescaleDB hypertable for position snapshots
   - Type-safe queries with sqlx (runtime type checking)

3. **stillwater-indexer** (`crates/indexer/`) - The Graph integration
   - GraphQL client for Uniswap v4 subgraph
   - Sync positions, pools, and swaps from blockchain
   - Automatic data conversion and insertion

4. **stillwater-analytics** (`crates/analytics/`) - P&L calculations
   - Fee estimation from swap volume
   - Impermanent loss formulas for concentrated liquidity
   - Position health status determination
   - Tick math utilities (price ↔ tick conversion)

5. **stillwater-api** (`crates/api/`) - REST API server
   - Axum web framework
   - Position endpoints with P&L and health data
   - Blockchain health checks
   - Data sync utility binary

## Prerequisites

- Rust (latest stable with 2024 edition)
- Docker and Docker Compose
- Just (command runner) - optional but recommended
- The Graph API key for Unichain Sepolia
- Alchemy API key for Unichain Sepolia RPC

## Getting Started

### 1. Clone the repository

```bash
git clone <repository-url>
cd stillwater
```

### 2. Set up environment variables

Create a `.env` file in the project root:

```env
# Database
POSTGRES_USER=admin
POSTGRES_PASSWORD=stillwater
POSTGRES_DB=stillwater
DATABASE_URL=postgres://admin:stillwater@localhost:5432/stillwater

# Cache
REDIS_URL=redis://localhost:6379

# Blockchain (Unichain Sepolia)
ETHEREUM_RPC_URL=https://unichain-sepolia.g.alchemy.com/v2/YOUR_ALCHEMY_KEY

# The Graph (Uniswap v4 on Unichain Sepolia)
GRAPH_API_URL=https://gateway.thegraph.com/api/YOUR_API_KEY/subgraphs/id/GWdEPuFDzBVc2EDC4grZkN5zecqKPYYP2okXd39fnE5R
```

**Note**: Replace `YOUR_ALCHEMY_KEY` and `YOUR_API_KEY` with your actual API keys.

### 3. Start Docker services

Using just:
```bash
cd docker
just dstart
```

Or using docker-compose directly:
```bash
docker-compose --env-file .env -f docker/docker-compose.yml up -d
```

### 4. Build and run the application

```bash
# Build all crates
cargo build

# Run the API server
cargo run -p stillwater-api
```

The server will start on `http://127.0.0.1:3000`

### 5. Sync blockchain data

```bash
# Note: Requires schema adaptation for Uniswap v4 (see TESTING.md)
cargo run -p stillwater-api --bin sync
```

## Project Structure

```
stillwater/
├── Cargo.toml                      # Workspace definition
├── CLAUDE.md                       # Claude Code project instructions
├── TESTING.md                      # Testing and deployment guide
├── crates/
│   ├── models/                     # Domain types & contracts
│   │   ├── src/
│   │   │   ├── pool.rs
│   │   │   ├── position.rs
│   │   │   ├── swap.rs
│   │   │   ├── snapshot.rs
│   │   │   ├── pnl.rs
│   │   │   ├── contracts/          # Uniswap v4 bindings
│   │   │   └── blockchain.rs
│   │   └── Cargo.toml
│   ├── db/                         # Database layer
│   │   ├── src/lib.rs              # CRUD operations
│   │   └── Cargo.toml
│   ├── indexer/                    # The Graph client
│   │   ├── src/
│   │   │   ├── queries.rs          # GraphQL queries
│   │   │   ├── types.rs            # Response types
│   │   │   └── lib.rs
│   │   └── Cargo.toml
│   ├── analytics/                  # P&L calculations
│   │   ├── src/
│   │   │   ├── pnl.rs
│   │   │   ├── health.rs
│   │   │   └── utils.rs
│   │   └── Cargo.toml
│   └── api/                        # REST API server
│       ├── src/
│       │   ├── main.rs
│       │   ├── state.rs
│       │   ├── config.rs
│       │   ├── handlers/
│       │   │   ├── mod.rs
│       │   │   └── positions.rs
│       │   └── bin/
│       │       └── sync.rs          # Data sync utility
│       └── Cargo.toml
├── migrations/                      # Database migrations
│   └── 001_initial_schema.sql
├── docker/
│   ├── docker-compose.yml           # PostgreSQL + Redis
│   └── justfile
└── contracts/                       # Uniswap v4 contracts
    └── lib/v4-core/
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run with output
cargo test -- --nocapture
```

### Code Formatting

The project uses rustfmt with custom configuration (100 char width, 4 spaces):

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

### Database Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations (also runs automatically on startup)
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

## Docker Commands

Using just (recommended):
```bash
cd docker
just dstart  # Start services
just dkill   # Stop services
```

Using docker-compose:
```bash
# Start services
docker-compose --env-file .env -f docker/docker-compose.yml up -d

# Stop services
docker-compose -f docker/docker-compose.yml down
```

## Architecture

Stillwater follows a layered architecture:

- **Entry Point**: Initializes services, runs migrations, and starts the Axum server
- **State Management**: Shared `AppState` containing database pool and Redis client
- **Services Layer**: Database and cache operations
- **Handlers**: HTTP request handlers using Axum extractors

The application uses Axum's state extraction pattern for dependency injection, allowing handlers to access shared resources like the database pool and Redis client.

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `POSTGRES_USER` | PostgreSQL username | `admin` |
| `POSTGRES_PASSWORD` | PostgreSQL password | `stillwater` |
| `POSTGRES_DB` | PostgreSQL database name | `stillwater` |
| `DATABASE_URL` | Full PostgreSQL connection string | `postgres://admin:stillwater@localhost:5432/stillwater` |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `ETHEREUM_RPC_URL` | Unichain Sepolia RPC endpoint | `https://unichain-sepolia.g.alchemy.com/v2/YOUR_KEY` |
| `GRAPH_API_URL` | The Graph API URL for Uniswap v4 | `https://gateway.thegraph.com/api/YOUR_KEY/subgraphs/id/...` |

## Current Status

### ✅ Completed
- Cargo workspace with 5 crates
- PostgreSQL 17 + TimescaleDB schema and migrations
- Database CRUD operations for all entities
- The Graph GraphQL client infrastructure
- P&L calculation engine (fees, IL, net P&L)
- Position health monitoring (Healthy/Warning/Critical)
- REST API with position endpoints
- Tick math utilities (price ↔ tick conversion)
- Data sync utility binary

### ⚠️ Known Issues & Next Steps

#### Uniswap v4 Schema Adaptation Required

The Uniswap v4 subgraph on Unichain Sepolia uses a different schema than v3:

**v3 Schema (Expected)**:
```graphql
type Position {
  id: ID!
  owner: String!
  pool: Pool!
  tickLower: BigInt!
  tickUpper: BigInt!
  liquidity: BigInt!
  transaction: Transaction!
}
```

**v4 Schema (Actual - Unichain Sepolia)**:
```graphql
type Position {
  id: ID!
  tokenId: BigInt!
  owner: String!
  origin: String!
  createdAtTimestamp: BigInt!
  subscriptions: [Subscribe!]!
  unsubscriptions: [Unsubscribe!]!
  transfers: [Transfer!]!
}

type ModifyLiquidity {
  id: ID!
  timestamp: BigInt!
  pool: Pool!
  tickLower: BigInt!
  tickUpper: BigInt!
  amount: BigInt!  # liquidity delta
  amount0: BigDecimal!
  amount1: BigDecimal!
  sender: Bytes
  origin: Bytes!
}
```

**What This Means**:
- v4 tracks liquidity via `ModifyLiquidity` events instead of `Position` entities with ticks
- Queries in `crates/indexer/src/queries.rs` need updating to:
  1. Query `ModifyLiquidity` events for tick ranges and liquidity
  2. Join `Position` (for owner/tokenId) with `ModifyLiquidity` (for pool/ticks/liquidity)
  3. Track liquidity changes over time by summing `amount` deltas per position

**Example Adapted Query**:
```graphql
query RecentModifyLiquidity($timestamp: BigInt!) {
  modifyLiquidities(
    where: { timestamp_gte: $timestamp }
    orderBy: timestamp
    orderDirection: desc
    first: 100
  ) {
    id
    timestamp
    pool {
      id
      token0 { id }
      token1 { id }
      fee
      tickSpacing
    }
    tickLower
    tickUpper
    amount
    amount0
    amount1
    origin
  }
}
```

#### P&L Calculation Simplifications (MVP)
- Fee estimation uses simplified 0.3% tier and 1% pool share assumption
- Production version should calculate exact fees from pool state
- Impermanent loss uses approximation formula

#### Recommended Enhancements
1. Update indexer queries for v4 `ModifyLiquidity` schema
2. Implement liquidity aggregation per position
3. Add exact fee calculation from pool state
4. Build frontend dashboard UI
5. Add webhook/notification system for health alerts
6. Implement CORS for frontend development

## API Endpoints

### Health & Status
- `GET /` - Root endpoint
- `GET /health` - Blockchain connection health check

### Position Tracking
- `GET /positions/{owner}` - Get all positions for an address
  - Returns: Array of positions with basic data

- `GET /positions/{owner}/{nft_id}?initial_price=X&current_price=Y&current_tick=Z&gas_spent=W`
  - Get position with complete P&L breakdown
  - Query params:
    - `initial_price`: Price when position was created (default: 1.0)
    - `current_price`: Current pool price (default: 1.0)
    - `current_tick`: Current tick (default: 0)
    - `gas_spent`: Total gas spent in decimal (default: 0)
  - Returns: Position data + P&L metrics (fees, IL, net P&L)

- `GET /positions/{owner}/{nft_id}/health?current_tick=X&initial_price=Y&current_price=Z&gas_spent=W`
  - Get position health status
  - Same query params as above
  - Returns: Health status (Healthy/Warning/Critical) with details

### Example Requests

```bash
# Check server health
curl http://127.0.0.1:3000/health

# Get positions for address
curl http://127.0.0.1:3000/positions/0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

# Get position P&L
curl "http://127.0.0.1:3000/positions/0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb/1?initial_price=1.0&current_price=1.05&current_tick=500&gas_spent=0.001"

# Get position health
curl "http://127.0.0.1:3000/positions/0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb/1/health?current_tick=500&initial_price=1.0&current_price=1.05&gas_spent=0.001"
```

## Database Schema

The database uses PostgreSQL 17 with TimescaleDB for time-series optimizations:

### Tables

- **pools** - Uniswap v4 pool configurations
  - pool_id, token0, token1, fee_tier, tick_spacing

- **positions** - User LP positions (represented as NFTs)
  - id, nft_id, owner, pool_id, tick_lower, tick_upper, liquidity, created_at

- **swaps** - Swap events for fee calculation
  - id, tx_hash, pool_id, amount0, amount1, timestamp

- **position_snapshots** - Time-series snapshots (TimescaleDB hypertable)
  - Hypertable partitioned by time for efficient historical queries
  - snapshot_time, position_id, liquidity, fees_earned, impermanent_loss, net_pnl

### P&L Calculation Details

**Fees Earned**:
- Estimated from swap volume
- MVP uses 0.3% fee tier and assumes 1% pool share
- Formula: `total_volume * 0.003 * 0.01`

**Impermanent Loss**:
- Calculated for concentrated liquidity positions
- Based on price movement and range width
- Formula: `price_change_pct / (1 + range_width) * 0.5`

**Net P&L**:
- Simple calculation: `fees_earned - impermanent_loss - gas_spent`

### Health Status Logic

Three health levels based on position state:

- **Healthy**: Position is in range AND has positive P&L
- **Warning**: Position is within 10% of range edge (may go out of range soon)
- **Critical**: Position is out of range OR has negative P&L

## Manual End-to-End Testing

### 1. Deploy Position on Unichain Sepolia

Use the Uniswap v4 interface or contracts to:
1. Create an LP position on a v4 pool
2. Note the position NFT ID and owner address
3. Make some swaps to generate fee activity

### 2. Sync Data from The Graph

```bash
# After adapting queries for v4 schema
cargo run -p stillwater-api --bin sync
```

### 3. Verify Database

```bash
psql $DATABASE_URL

# Check synced data
SELECT * FROM positions;
SELECT * FROM pools;
SELECT * FROM swaps;
```

### 4. Query API with Real Data

```bash
# List positions for your address
curl "http://127.0.0.1:3000/positions/{your_address}"

# Get P&L for specific position
curl "http://127.0.0.1:3000/positions/{your_address}/{nft_id}?initial_price={price_at_creation}&current_price={current_price}&current_tick={current_tick}&gas_spent={total_gas}"

# Check health status
curl "http://127.0.0.1:3000/positions/{your_address}/{nft_id}/health?current_tick={current_tick}&initial_price={price_at_creation}&current_price={current_price}&gas_spent={total_gas}"
```

### 5. Acceptance Criteria

- ✅ API returns position within 30 seconds of sync completing
- ⚠️  P&L calculation uses simplified formulas (acceptable for MVP)
- ✅ Health status updates correctly based on tick position and P&L
- ⚠️  Requires schema adaptation for v4 ModifyLiquidity events

## Troubleshooting

### Database Connection Errors

```bash
# Check if Docker containers are running
docker ps | grep -E "(postgres|redis)"

# Restart containers if needed
cd docker && just dkill && just dstart
```

### Migration Errors

```bash
# Revert last migration
sqlx migrate revert

# Rerun all migrations
sqlx migrate run
```

### The Graph API Errors

**Check API key**: Verify `GRAPH_API_URL` in `.env` is correct

**Test subgraph health**:
```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '{"query":"{ _meta { block { number } } }"}' \
  $GRAPH_API_URL
```

**Schema mismatch**: Ensure queries match v4 schema (see "Known Issues" section above)

### Server Won't Start

- Check `.env` file exists and has all required variables
- Ensure Docker services are running
- Check port 3000 isn't already in use: `lsof -i :3000`

## Performance Notes

- **TimescaleDB hypertable** on `position_snapshots` for efficient time-series queries
- **PostgreSQL connection pool** limited to 5 connections (configurable in code)
- **Swap queries** default to 24-hour lookback window to limit data volume
- **Position sync** fetches last 1 hour of activity per sync run

## License

MIT
