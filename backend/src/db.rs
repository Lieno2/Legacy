use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn create_pg_pool(url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(url)
        .await
        .expect("Failed to connect to PostgreSQL")
}

pub fn create_redis_client(url: &str) -> redis::Client {
    redis::Client::open(url).expect("Failed to create Redis client")
}
