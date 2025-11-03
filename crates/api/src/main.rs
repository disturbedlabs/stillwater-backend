mod config;
mod state;

use axum::{Router, extract::State, routing::get};
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();
    config::init_tracing();

    let db_pool = config::init_database().await;
    sqlx::migrate!("../../migrations").run(&db_pool).await.expect("Failed to run migrations");
    info!("Database migrations completed successfully");

    let redis_client = config::init_redis();

    let blockchain = config::init_blockchain();
    info!("Blockchain service initialized");

    let app_state = AppState::new(db_pool, redis_client, blockchain);

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server running on http://{}", addr);

    let listener = TcpListener::bind(addr).await.expect("Failed to bind TCP listener");
    axum::serve(listener, app.into_make_service()).await.expect("Failed to start server");
}

async fn root_handler(State(_state): State<AppState>) -> &'static str {
    "Stillwater API is running"
}

async fn health_handler(State(state): State<AppState>) -> String {
    match state.blockchain.get_block_number().await {
        Ok(block_number) => format!("Healthy. Latest block: {}", block_number),
        Err(e) => format!("Unhealthy. Error: {}", e),
    }
}
