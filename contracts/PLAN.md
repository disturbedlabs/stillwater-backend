# Stillwater MVP Development Plan

## Context

Building a professional liquidity management platform for Uniswap v4 LPs on Base. Core value: accurate P&L tracking (fees - IL - gas) with proactive warnings.

## Tech Stack

- **Language:** Rust (Cargo workspace)
- **Framework:** axum (web), sqlx (database), alloy (Ethereum)
- **Database:** Postgres with TimescaleDB extension
- **Chain:** Base Sepolia (testnet)
- **Data Source:** The Graph subgraph (Uniswap v4)

## Architecture

Modular monolith with 5 crates:

1. `indexer` - Fetch position/swap data from The Graph
2. `analytics` - P&L, IL, health calculations
3. `models` - Shared domain types
4. `db` - Database layer with sqlx
5. `api` - REST API with axum

## Phase 1 Tasks (Priority Order)

### Task 1: Project Setup

Create Cargo workspace with 5 crates following this structure:

```
stillwater/
├── crates/
│   ├── indexer/
│   ├── analytics/
│   ├── models/
│   ├── db/
│   └── api/
├── migrations/
├── Cargo.toml (workspace)
└── docker-compose.yml
```

**Acceptance Criteria:**

- All crates compile with `cargo build`
- Workspace dependencies defined once in root Cargo.toml
- docker-compose.yml has Postgres 16 + TimescaleDB extension

---

### Task 2: Database Schema

Design schema for tracking LP positions, swaps, and pools.

**Tables Needed:**

- `pools` (pool_id, token0, token1, fee_tier, tick_spacing)
- `positions` (nft_id, owner, pool_id, tick_lower, tick_upper, liquidity, created_at)
- `swaps` (tx_hash, pool_id, amount0, amount1, timestamp)
- `position_snapshots` (position_id, timestamp, fees_earned, liquidity, price)

**Acceptance Criteria:**

- Migration file in `migrations/001_initial_schema.sql`
- TimescaleDB hypertable on `position_snapshots` for time-series
- Runs cleanly with `sqlx migrate run`

---

### Task 3: Models Crate

Define Rust types for domain entities with proper derives.

**Types:**

```rust
// models/src/position.rs
pub struct Position {
    pub nft_id: String,
    pub owner: String,
    pub pool_id: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: U256,
    pub created_at: DateTime<Utc>,
}

// models/src/pnl.rs
pub struct PositionPnL {
    pub fees_earned: Decimal,
    pub impermanent_loss: Decimal,
    pub gas_spent: Decimal,
    pub net_pnl: Decimal,
}

pub enum HealthStatus {
    Healthy,   // In range, good fees
    Warning,   // Near out of range
    Critical,  // Out of range or negative P&L
}
```

**Acceptance Criteria:**

- All types are `Serialize` + `Deserialize` + `Clone`
- Use proper types (U256 for on-chain, Decimal for calcs)
- Re-exported cleanly from `models/src/lib.rs`

---

### Task 4: Database Crate

Build DB layer with connection pooling and basic queries.

**Functions:**

```rust
// db/src/lib.rs
pub async fn get_pool() -> Result<PgPool>
pub async fn insert_position(pool: &PgPool, pos: &Position) -> Result<()>
pub async fn get_position_by_nft(pool: &PgPool, nft_id: &str) -> Result<Option<Position>>
pub async fn insert_swap(pool: &PgPool, swap: &Swap) -> Result<()>
pub async fn get_swaps_for_pool(pool: &PgPool, pool_id: &str, since: DateTime<Utc>) -> Result<Vec<Swap>>
```

**Acceptance Criteria:**

- Uses sqlx with compile-time checked queries
- Proper error handling (anyhow or custom Result)
- Connection pool configured (max 10 connections)

---

### Task 5: Indexer Crate (The Graph Client)

Query The Graph subgraph for Uniswap v4 positions and swaps.

**Implementation:**

- Use reqwest for GraphQL queries
- Query positions by owner or pool
- Query recent swaps
- Poll every 30 seconds in background task

**Acceptance Criteria:**

- Successfully fetches positions from Base Sepolia subgraph
- Inserts new positions into DB
- Handles pagination for large result sets
- Graceful error handling + retry logic

---

### Task 6: Analytics Crate (P&L Calculation)

Implement CORRECT P&L math for concentrated liquidity positions.

**Core Functions:**

```rust
// analytics/src/pnl.rs
pub fn calculate_fees_earned(position: &Position, swaps: &[Swap]) -> Decimal
pub fn calculate_impermanent_loss(position: &Position, initial_price: Decimal, current_price: Decimal) -> Decimal
pub fn calculate_net_pnl(fees: Decimal, il: Decimal, gas: Decimal) -> Decimal

// analytics/src/health.rs
pub fn get_position_health(position: &Position, current_tick: i32) -> HealthStatus
```

**Acceptance Criteria:**

- P&L matches manual calculation for test position
- IL formula correct for concentrated liquidity
- Health status logic:
  - Healthy = in range + positive P&L
  - Warning = within 10% of range edge
  - Critical = out of range OR negative P&L

---

### Task 7: API Crate (REST Endpoints)

Build axum API with health check and position endpoints.

**Endpoints:**

```
GET /health                      → {status: "ok"}
GET /positions/:nft_id           → Full position + P&L
GET /positions/:nft_id/health    → Health status
GET /positions?owner=0x...       → List positions by owner
```

**Acceptance Criteria:**

- Server runs on port 3000
- Proper error responses (404, 500)
- CORS enabled for local frontend development
- JSON responses with proper content-type

---

### Task 8: End-to-End Test

Deploy to testnet, create real position, verify tracking.

**Steps:**

1. Deploy to Base Sepolia
2. Create LP position on v4 pool
3. Make some swaps to generate fees
4. Query API and verify P&L is accurate

**Acceptance Criteria:**

- API returns position within 30 seconds of creation
- P&L calculation matches etherscan data
- Health status updates correctly

---

## Development Guidelines

**Code Quality:**

- Use `rustfmt` (run on every commit)
- Run `clippy` and fix all warnings
- Write integration tests for P&L math
- Document public functions

**Error Handling:**

- Use `anyhow::Result` for application errors
- Never panic in production code
- Log errors with context

**Database:**

- All timestamps in UTC
- Use transactions for multi-step operations
- Index foreign keys

**Performance:**

- Database queries < 100ms
- API responses < 200ms
- Use connection pooling

## Out of Scope for MVP

- Custom indexer (use The Graph)
- Frontend (API only)
- Multi-chain support
- Automated rebalancing
- Price predictions
- User authentication

## Success Criteria

One LP position tracked end-to-end with accurate P&L, deployable API, solid foundation for Phase 2 (frontend + advanced features).
