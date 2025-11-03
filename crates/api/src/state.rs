use redis::Client as RedisClient;
use sqlx::PgPool;
use stillwater_models::BlockchainService;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: RedisClient,
    pub blockchain: BlockchainService,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_client: RedisClient, blockchain: BlockchainService) -> Self {
        Self {
            db_pool,
            redis_client,
            blockchain,
        }
    }
}
