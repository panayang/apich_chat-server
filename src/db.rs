
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub type DbPool = Pool<Postgres>;

pub async fn connect(database_url: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
}
