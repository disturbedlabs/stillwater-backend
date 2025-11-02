use redis::Client as RedisClient;
use sqlx::PgPool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: RedisClient,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_client: RedisClient) -> Self {
        Self {
            db_pool,
            redis_client,
        }
    }
}
