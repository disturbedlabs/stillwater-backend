use alloy::primitives::U256;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Position snapshot for time-series P&L tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionSnapshot {
    pub id: i64,
    pub position_id: i64,
    pub timestamp: DateTime<Utc>,
    pub fees_earned: Decimal,
    #[serde(with = "u256_serde")]
    pub liquidity: U256,
    pub price: Decimal,
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
