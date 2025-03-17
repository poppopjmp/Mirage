use crate::config::{DatabaseConfig, RedisConfig};
use crate::models::UserModel;
use mirage_common::{Error, Result};
use redis::{Client as RedisClient, AsyncCommands};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

/// Create database connection pool
pub async fn create_db_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| Error::Database(format!("Database connection failed: {}", e)))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| Error::Database(format!("Migration failed: {}", e)))?;

    Ok(pool)
}

/// Create Redis client
pub fn create_redis_client(config: &RedisConfig) -> Result<RedisClient> {
    RedisClient::open(config.url.clone())
        .map_err(|e| Error::Database(format!("Redis connection failed: {}", e)))
}

/// User repository for database operations
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, user: &UserModel) -> Result<UserModel> {
        let record = sqlx::query_as!(
            UserModel,
            r#"
            INSERT INTO users (id, username, email, password_hash, roles, is_active)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, username, email, password_hash, roles as "roles: Vec<String>", is_active, created_at, updated_at
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            &user.roles as _,
            user.is_active
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create user: {}", e)))?;

        Ok(record)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserModel>> {
        let record = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, password_hash, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find user: {}", e)))?;

        Ok(record)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserModel>> {
        let record = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, password_hash, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find user: {}", e)))?;

        Ok(record)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserModel>> {
        let record = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, password_hash, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find user: {}", e)))?;

        Ok(record)
    }
}

/// Token repository for token operations (Redis)
pub struct TokenRepository {
    client: RedisClient,
}

impl TokenRepository {
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }

    pub async fn store_refresh_token(&self, token_id: &str, user_id: &Uuid, ttl: i64) -> Result<()> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;

        conn.set_ex(format!("refresh_token:{}", token_id), user_id.to_string(), ttl as usize)
            .await
            .map_err(|e| Error::Database(format!("Failed to store refresh token: {}", e)))?;

        Ok(())
    }

    pub async fn get_refresh_token(&self, token_id: &str) -> Result<Option<String>> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;

        let result: Option<String> = conn.get(format!("refresh_token:{}", token_id))
            .await
            .map_err(|e| Error::Database(format!("Failed to get refresh token: {}", e)))?;

        Ok(result)
    }

    pub async fn invalidate_refresh_token(&self, token_id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;

        conn.del(format!("refresh_token:{}", token_id))
            .await
            .map_err(|e| Error::Database(format!("Failed to invalidate refresh token: {}", e)))?;

        Ok(())
    }
}
