//! Database utilities and connections

use sqlx::PgPool;
use crate::error::{Error, Result};

pub type DatabasePool = PgPool;

pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<DatabasePool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .map_err(|e| Error::Database(e.to_string()))
}

pub async fn run_migrations(pool: &DatabasePool, migration_path: &str) -> Result<()> {
    // This would typically use sqlx::migrate! macro
    // For now, just a placeholder
    Ok(())
}

pub async fn health_check(pool: &DatabasePool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
    Ok(())
}