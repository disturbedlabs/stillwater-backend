use axum::{Router, extract::State, routing::get};
use dotenv::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;
use stillwater::{config, state::AppState};

#[tokio::main]
async fn main() {
    dotenv().ok();
    config::init_tracing();

    let db_pool = config::init_database().await;
    sqlx::migrate!().run(&db_pool).await.expect("Failed to run migrations");
    info!("Database migrations completed successfully");

    let redis_client = config::init_redis();

    let app_state = AppState::new(db_pool, redis_client);

    let app = Router::new()
        .route("/", get(root_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Server running on http://{}", addr);

    let listener = TcpListener::bind(addr).await.expect("Failed to bind TCP listener");
    axum::serve(listener, app.into_make_service()).await.expect("Failed to start server");
}

async fn root_handler(State(_state): State<AppState>) -> &'static str {
    "Stillwater API is running"
}
