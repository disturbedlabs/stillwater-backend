use redis::Client as RedisClient;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tracing_subscriber::EnvFilter;

/// Initializes tracing (logging)
pub fn init_tracing() {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::new("info")).init();
}

/// Initializes PostgreSQL connection pool
pub async fn init_database() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create Postgres pool")
}

/// Initializes Redis client
pub fn init_redis() -> RedisClient {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set in .env");
    RedisClient::open(redis_url).expect("Failed to create Redis client")
}
