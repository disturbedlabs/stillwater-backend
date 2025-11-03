use alloy::primitives::I256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Swap event for fee calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swap {
    pub id: i64,
    pub tx_hash: String,
    pub pool_id: String,
    #[serde(with = "i256_serde")]
    pub amount0: I256,
    #[serde(with = "i256_serde")]
    pub amount1: I256,
    pub timestamp: DateTime<Utc>,
}

// Custom serialization for I256
mod i256_serde {
    use alloy::primitives::I256;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &I256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<I256, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<I256>().map_err(serde::de::Error::custom)
    }
}
