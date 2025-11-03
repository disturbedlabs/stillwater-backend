use alloy::primitives::{I256, U256};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use stillwater_models::{Pool, Position, PositionSnapshot, Swap};

pub type DbPool = PgPool;

/// Create a PostgreSQL connection pool
pub async fn get_pool() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .context("DATABASE_URL must be set in environment")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .context("Failed to create database connection pool")?;

    Ok(pool)
}

// ============================================================================
// Pool Operations
// ============================================================================

/// Insert a new pool
pub async fn insert_pool(pool: &PgPool, p: &Pool) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO pools (pool_id, token0, token1, fee_tier, tick_spacing, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (pool_id) DO NOTHING
        "#,
    )
    .bind(&p.pool_id)
    .bind(&p.token0)
    .bind(&p.token1)
    .bind(p.fee_tier)
    .bind(p.tick_spacing)
    .bind(p.created_at)
    .execute(pool)
    .await
    .context("Failed to insert pool")?;

    Ok(())
}

/// Get a pool by pool_id
pub async fn get_pool_by_id(pool: &PgPool, pool_id: &str) -> Result<Option<Pool>> {
    let result = sqlx::query_as::<_, Pool>(
        r#"
        SELECT pool_id, token0, token1, fee_tier, tick_spacing, created_at
        FROM pools
        WHERE pool_id = $1
        "#,
    )
    .bind(pool_id)
    .fetch_optional(pool)
    .await
    .context("Failed to get pool by ID")?;

    Ok(result)
}

// ============================================================================
// Position Operations
// ============================================================================

/// Insert a new position
pub async fn insert_position(pool: &PgPool, pos: &Position) -> Result<()> {
    let liquidity_str = pos.liquidity.to_string();

    sqlx::query(
        r#"
        INSERT INTO positions (nft_id, owner, pool_id, tick_lower, tick_upper, liquidity, created_at)
        VALUES ($1, $2, $3, $4, $5, $6::numeric, $7)
        ON CONFLICT (nft_id) DO NOTHING
        "#,
    )
    .bind(&pos.nft_id)
    .bind(&pos.owner)
    .bind(&pos.pool_id)
    .bind(pos.tick_lower)
    .bind(pos.tick_upper)
    .bind(&liquidity_str)
    .bind(pos.created_at)
    .execute(pool)
    .await
    .context("Failed to insert position")?;

    Ok(())
}

/// Get a position by database ID
pub async fn get_position_by_id(pool: &PgPool, id: i64) -> Result<Option<Position>> {
    let row = sqlx::query(
        r#"
        SELECT id, nft_id, owner, pool_id, tick_lower, tick_upper, liquidity::text, created_at
        FROM positions
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("Failed to get position by ID")?;

    Ok(row.map(|r| {
        let liquidity_str: String = r.get(6);
        Position {
            id: r.get(0),
            nft_id: r.get(1),
            owner: r.get(2),
            pool_id: r.get(3),
            tick_lower: r.get(4),
            tick_upper: r.get(5),
            liquidity: U256::from_str_radix(&liquidity_str, 10).unwrap_or_default(),
            created_at: r.get(7),
        }
    }))
}

/// Get a position by NFT ID
pub async fn get_position_by_nft(pool: &PgPool, nft_id: &str) -> Result<Option<Position>> {
    let row = sqlx::query(
        r#"
        SELECT id, nft_id, owner, pool_id, tick_lower, tick_upper, liquidity::text, created_at
        FROM positions
        WHERE nft_id = $1
        "#,
    )
    .bind(nft_id)
    .fetch_optional(pool)
    .await
    .context("Failed to get position by NFT ID")?;

    Ok(row.map(|r| {
        let liquidity_str: String = r.get(6);
        Position {
            id: r.get(0),
            nft_id: r.get(1),
            owner: r.get(2),
            pool_id: r.get(3),
            tick_lower: r.get(4),
            tick_upper: r.get(5),
            liquidity: U256::from_str_radix(&liquidity_str, 10).unwrap_or_default(),
            created_at: r.get(7),
        }
    }))
}

/// Get all positions for an owner
pub async fn get_positions_by_owner(pool: &PgPool, owner: &str) -> Result<Vec<Position>> {
    let rows = sqlx::query(
        r#"
        SELECT id, nft_id, owner, pool_id, tick_lower, tick_upper, liquidity::text, created_at
        FROM positions
        WHERE owner = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(owner)
    .fetch_all(pool)
    .await
    .context("Failed to get positions by owner")?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let liquidity_str: String = r.get(6);
            Position {
                id: r.get(0),
                nft_id: r.get(1),
                owner: r.get(2),
                pool_id: r.get(3),
                tick_lower: r.get(4),
                tick_upper: r.get(5),
                liquidity: U256::from_str_radix(&liquidity_str, 10).unwrap_or_default(),
                created_at: r.get(7),
            }
        })
        .collect())
}

/// Get all positions in a pool
pub async fn get_positions_by_pool(pool: &PgPool, pool_id: &str) -> Result<Vec<Position>> {
    let rows = sqlx::query(
        r#"
        SELECT id, nft_id, owner, pool_id, tick_lower, tick_upper, liquidity::text, created_at
        FROM positions
        WHERE pool_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(pool_id)
    .fetch_all(pool)
    .await
    .context("Failed to get positions by pool")?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let liquidity_str: String = r.get(6);
            Position {
                id: r.get(0),
                nft_id: r.get(1),
                owner: r.get(2),
                pool_id: r.get(3),
                tick_lower: r.get(4),
                tick_upper: r.get(5),
                liquidity: U256::from_str_radix(&liquidity_str, 10).unwrap_or_default(),
                created_at: r.get(7),
            }
        })
        .collect())
}

// ============================================================================
// Swap Operations
// ============================================================================

/// Insert a new swap
pub async fn insert_swap(pool: &PgPool, swap: &Swap) -> Result<()> {
    let amount0_str = swap.amount0.to_string();
    let amount1_str = swap.amount1.to_string();

    sqlx::query(
        r#"
        INSERT INTO swaps (tx_hash, pool_id, amount0, amount1, timestamp)
        VALUES ($1, $2, $3::numeric, $4::numeric, $5)
        ON CONFLICT (tx_hash, pool_id) DO NOTHING
        "#,
    )
    .bind(&swap.tx_hash)
    .bind(&swap.pool_id)
    .bind(&amount0_str)
    .bind(&amount1_str)
    .bind(swap.timestamp)
    .execute(pool)
    .await
    .context("Failed to insert swap")?;

    Ok(())
}

/// Get swaps for a pool since a specific timestamp
pub async fn get_swaps_for_pool(
    pool: &PgPool,
    pool_id: &str,
    since: DateTime<Utc>,
) -> Result<Vec<Swap>> {
    let rows = sqlx::query(
        r#"
        SELECT id, tx_hash, pool_id, amount0::text, amount1::text, timestamp
        FROM swaps
        WHERE pool_id = $1 AND timestamp >= $2
        ORDER BY timestamp ASC
        "#,
    )
    .bind(pool_id)
    .bind(since)
    .fetch_all(pool)
    .await
    .context("Failed to get swaps for pool")?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let amount0_str: String = r.get(3);
            let amount1_str: String = r.get(4);
            Swap {
                id: r.get(0),
                tx_hash: r.get(1),
                pool_id: r.get(2),
                amount0: amount0_str.parse::<I256>().unwrap_or_default(),
                amount1: amount1_str.parse::<I256>().unwrap_or_default(),
                timestamp: r.get(5),
            }
        })
        .collect())
}

// ============================================================================
// Snapshot Operations
// ============================================================================

/// Insert a new position snapshot
pub async fn insert_snapshot(pool: &PgPool, snapshot: &PositionSnapshot) -> Result<()> {
    let liquidity_str = snapshot.liquidity.to_string();

    sqlx::query(
        r#"
        INSERT INTO position_snapshots (position_id, timestamp, fees_earned, liquidity, price)
        VALUES ($1, $2, $3, $4::numeric, $5)
        "#,
    )
    .bind(snapshot.position_id)
    .bind(snapshot.timestamp)
    .bind(snapshot.fees_earned)
    .bind(&liquidity_str)
    .bind(snapshot.price)
    .execute(pool)
    .await
    .context("Failed to insert position snapshot")?;

    Ok(())
}

/// Get snapshots for a position in a time range
pub async fn get_snapshots_for_position(
    pool: &PgPool,
    position_id: i64,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<PositionSnapshot>> {
    let rows = sqlx::query(
        r#"
        SELECT id, position_id, timestamp, fees_earned, liquidity::text, price
        FROM position_snapshots
        WHERE position_id = $1 AND timestamp >= $2 AND timestamp <= $3
        ORDER BY timestamp ASC
        "#,
    )
    .bind(position_id)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await
    .context("Failed to get snapshots for position")?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let liquidity_str: String = r.get(4);
            PositionSnapshot {
                id: r.get(0),
                position_id: r.get(1),
                timestamp: r.get(2),
                fees_earned: r.get(3),
                liquidity: U256::from_str_radix(&liquidity_str, 10).unwrap_or_default(),
                price: r.get(5),
            }
        })
        .collect())
}
