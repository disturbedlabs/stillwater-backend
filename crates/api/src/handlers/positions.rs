use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use stillwater_analytics::{
    calculate_position_pnl, get_health_details, get_position_health, is_in_range,
};
use stillwater_db::{get_position_by_nft, get_positions_by_owner, get_swaps_for_pool};
use stillwater_models::PositionPnL;
use tracing::{error, info};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct PositionResponse {
    pub nft_id: String,
    pub owner: String,
    pub pool_id: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PositionWithPnlResponse {
    pub nft_id: String,
    pub owner: String,
    pub pool_id: String,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: String,
    pub created_at: String,
    pub pnl: PositionPnL,
    pub in_range: bool,
    pub current_tick: i32,
}

#[derive(Debug, Serialize)]
pub struct PositionHealthResponse {
    pub nft_id: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Deserialize)]
pub struct PnlQueryParams {
    #[serde(default = "default_initial_price")]
    pub initial_price: String,
    #[serde(default = "default_current_price")]
    pub current_price: String,
    #[serde(default = "default_current_tick")]
    pub current_tick: i32,
    #[serde(default = "default_gas_spent")]
    pub gas_spent: String,
}

fn default_initial_price() -> String {
    "1.0".to_string()
}

fn default_current_price() -> String {
    "1.0".to_string()
}

fn default_current_tick() -> i32 {
    0
}

fn default_gas_spent() -> String {
    "0".to_string()
}

/// GET /positions/:owner
/// Get all positions for an address
pub async fn get_positions_handler(
    State(state): State<AppState>,
    Path(owner): Path<String>,
) -> impl IntoResponse {
    info!("Fetching positions for owner: {}", owner);

    match get_positions_by_owner(&state.db_pool, &owner).await {
        Ok(positions) => {
            let response: Vec<PositionResponse> = positions
                .into_iter()
                .map(|p| PositionResponse {
                    nft_id: p.nft_id,
                    owner: p.owner,
                    pool_id: p.pool_id,
                    tick_lower: p.tick_lower,
                    tick_upper: p.tick_upper,
                    liquidity: p.liquidity.to_string(),
                    created_at: p.created_at.to_rfc3339(),
                })
                .collect();

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("Failed to fetch positions: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(vec![]),
            )
        }
    }
}

/// GET /positions/:owner/:nft_id?initial_price=X&current_price=Y&current_tick=Z&gas_spent=W
/// Get specific position with P&L
pub async fn get_position_with_pnl_handler(
    State(state): State<AppState>,
    Path((owner, nft_id)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<PnlQueryParams>,
) -> impl IntoResponse {
    info!("Fetching position {} for owner {} with P&L", nft_id, owner);

    // Get position from database
    let position = match get_position_by_nft(&state.db_pool, &nft_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Position not found" })),
            )
        }
        Err(e) => {
            error!("Failed to fetch position: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server error" })),
            );
        }
    };

    // Verify owner matches
    if position.owner.to_lowercase() != owner.to_lowercase() {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Position does not belong to this owner" })),
        );
    }

    // Get swaps for the pool from the past 24 hours
    let since = Utc::now() - chrono::Duration::hours(24);
    let swaps = match get_swaps_for_pool(&state.db_pool, &position.pool_id, since).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to fetch swaps: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to fetch swaps" })),
            );
        }
    };

    // Parse price parameters
    let initial_price = match params.initial_price.parse::<Decimal>() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid initial_price parameter" })),
            );
        }
    };

    let current_price = match params.current_price.parse::<Decimal>() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid current_price parameter" })),
            );
        }
    };

    let gas_spent = match params.gas_spent.parse::<Decimal>() {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid gas_spent parameter" })),
            );
        }
    };

    // Calculate P&L
    let pnl = calculate_position_pnl(
        &position,
        &swaps,
        initial_price,
        current_price,
        gas_spent,
    );

    let in_range = is_in_range(params.current_tick, position.tick_lower, position.tick_upper);

    let response = PositionWithPnlResponse {
        nft_id: position.nft_id,
        owner: position.owner,
        pool_id: position.pool_id,
        tick_lower: position.tick_lower,
        tick_upper: position.tick_upper,
        liquidity: position.liquidity.to_string(),
        created_at: position.created_at.to_rfc3339(),
        pnl,
        in_range,
        current_tick: params.current_tick,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
}

/// GET /positions/:owner/:nft_id/health?current_tick=X&initial_price=Y&current_price=Z&gas_spent=W
/// Get position health status
pub async fn get_position_health_handler(
    State(state): State<AppState>,
    Path((owner, nft_id)): Path<(String, String)>,
    axum::extract::Query(params): axum::extract::Query<PnlQueryParams>,
) -> impl IntoResponse {
    info!("Fetching health for position {} owner {}", nft_id, owner);

    // Get position from database
    let position = match get_position_by_nft(&state.db_pool, &nft_id).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Position not found" })),
            )
        }
        Err(e) => {
            error!("Failed to fetch position: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Internal server error" })),
            );
        }
    };

    // Verify owner matches
    if position.owner.to_lowercase() != owner.to_lowercase() {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Position does not belong to this owner" })),
        );
    }

    // Get swaps for the pool from the past 24 hours
    let since = Utc::now() - chrono::Duration::hours(24);
    let swaps = match get_swaps_for_pool(&state.db_pool, &position.pool_id, since).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to fetch swaps: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to fetch swaps" })),
            );
        }
    };

    // Parse price parameters
    let initial_price = match params.initial_price.parse::<Decimal>() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid initial_price parameter" })),
            );
        }
    };

    let current_price = match params.current_price.parse::<Decimal>() {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid current_price parameter" })),
            );
        }
    };

    let gas_spent = match params.gas_spent.parse::<Decimal>() {
        Ok(g) => g,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid gas_spent parameter" })),
            );
        }
    };

    // Calculate P&L
    let pnl = calculate_position_pnl(
        &position,
        &swaps,
        initial_price,
        current_price,
        gas_spent,
    );

    // Get health status
    let status = get_position_health(&position, params.current_tick, &pnl);
    let details = get_health_details(&position, params.current_tick, &pnl);

    let response = PositionHealthResponse {
        nft_id: position.nft_id,
        status: format!("{:?}", status),
        details,
    };

    (StatusCode::OK, Json(serde_json::to_value(response).unwrap()))
}
