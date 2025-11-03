use alloy::primitives::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// LP position NFT
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub id: i64,
    pub nft_id: String,
    pub owner: String,
    pub pool_id: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    #[serde(with = "u256_serde")]
    pub liquidity: U256,
    pub created_at: DateTime<Utc>,
}

// Custom serialization for U256
mod u256_serde {
    use alloy::primitives::U256;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        U256::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
    }
}
