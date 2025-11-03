use anyhow::Result;
use dotenv::dotenv;
use sqlx::PgPool;
use stillwater_indexer::GraphIndexer;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting position sync from The Graph...");

    // Get database URL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment");

    // Connect to database
    let db_pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    info!("Connected to database");

    // Create indexer
    let indexer = GraphIndexer::from_env()
        .expect("Failed to create GraphIndexer. Ensure GRAPH_API_URL is set");

    info!("Indexer initialized with Graph API URL");

    // Sync positions
    match indexer.sync_positions(&db_pool).await {
        Ok(count) => {
            info!("✓ Successfully synced {} positions", count);
        }
        Err(e) => {
            error!("✗ Failed to sync positions: {}", e);
            return Err(e);
        }
    }

    info!("Sync completed successfully!");

    Ok(())
}
