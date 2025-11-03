use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Uniswap v4 pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    pub pool_id: String,
    pub token0: String,
    pub token1: String,
    pub fee_tier: i32,
    pub tick_spacing: i32,
    pub created_at: DateTime<Utc>,
}
