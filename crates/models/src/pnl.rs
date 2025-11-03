use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// P&L breakdown for a position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionPnL {
    pub fees_earned: Decimal,
    pub impermanent_loss: Decimal,
    pub gas_spent: Decimal,
    pub net_pnl: Decimal,
}

/// Health status of a position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// In range, positive P&L
    Healthy,
    /// Near out of range (within 10% of range edge)
    Warning,
    /// Out of range or negative P&L
    Critical,
}

impl HealthStatus {
    /// Get a human-readable description of the health status
    pub fn description(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "Position is in range with positive P&L",
            HealthStatus::Warning => "Position is near the edge of its range",
            HealthStatus::Critical => "Position is out of range or has negative P&L",
        }
    }
}
