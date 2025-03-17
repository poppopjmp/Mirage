use crate::models::{LoginResult, RefreshResult, UserModel};
use crate::repositories::{DbPool, TokenRepository, UserRepository};
use crate::jwt::{self, validate_refresh_token};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::errors::Error as JwtError;
use mirage_common::{Error, Result, models::User};
use redis::Client as RedisClient;
use sqlx::Postgres;
use uuid::Uuid;

pub use crate::jwt::{generate_token, generate_refresh_token};

pub struct AuthService {
    user_repo: UserRepository,
    token_repo: TokenRepository,
}

impl AuthService {
    pub fn new(pool: DbPool, redis_client: RedisClient) -> Self {
        Self {
            user_repo: UserRepository::new(pool),
            token_repo: TokenRepository::new(redis_client),
        }
    }

    pub async fn register(&self, username: &str, email: &str, password: &str) -> Result<User> {
        // Check if username already exists
        if let Some(_) = self.user_repo.find_by_username(username).await? {
            return Err(Error::Validation("Username already taken".to_string()));
        }

        // Check if email already exists
        if let Some(_) = self.user_repo.find_by_email(email).await? {
            return Err(Error::Validation("Email already registered".to_string()));
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| Error::Internal(format!("Failed to hash password: {}", e)))?
            .to_string();

        // Create user
        let user = UserModel {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            roles: vec!["user".to_string()],
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_user = self.user_repo.create_user(&user).await?;
        Ok(created_user.into())
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<LoginResult> {
        // Find user by username
        let user = self
            .user_repo
            .find_by_username(username)
            .await?
            .ok_or_else(|| Error::Authentication("Invalid username or password".to_string()))?;

        // Verify password
        let parsed_hash = PasswordHash::new(&user.password_hash)
            .map_err(|e| Error::Internal(format!("Failed to parse password hash: {}", e)))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| Error::Authentication("Invalid username or password".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(Error::Authorization("Account is disabled".to_string()));
        }

        Ok(LoginResult {
            user_id: user.id,
            roles: user.roles,
        })
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<RefreshResult> {
        // Parse token without validation to get the JTI
        let claims = match validate_refresh_token(refresh_token, "dummy_secret_for_parsing") {
            Ok(claims) => claims,
            Err(JwtError::InvalidSignature) => {
                // This is expected since we used a dummy secret
                // Extract claims manually for JTI
                let parts: Vec<&str> = refresh_token.split('.').collect();
                if parts.len() != 3 {
                    return Err(Error::Authentication("Invalid token format".to_string()));
                }
                
                let payload = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
                    .map_err(|_| Error::Authentication("Invalid token payload".to_string()))?;
                
                let claims: crate::jwt::RefreshClaims = serde_json::from_slice(&payload)
                    .map_err(|_| Error::Authentication("Invalid token claims".to_string()))?;
                
                claims
            }
            Err(e) => return Err(Error::Authentication(format!("Invalid refresh token: {}", e))),
        };

        // Get user ID from Redis using JTI
        let user_id_str = self
            .token_repo
            .get_refresh_token(&claims.jti)
            .await?
            .ok_or_else(|| Error::Authentication("Refresh token not found or expired".to_string()))?;

        let user_id = Uuid::parse_str(&user_id_str)
            .map_err(|_| Error::Internal("Invalid user ID format in token store".to_string()))?;

        // Get user from database
        let user = self
            .user_repo
            .find_by_id(&user_id)
            .await?
            .ok_or_else(|| Error::NotFound("User not found".to_string()))?;

        // Check if user is active
        if !user.is_active {
            return Err(Error::Authorization("Account is disabled".to_string()));
        }

        Ok(RefreshResult {
            user_id: user.id,
            roles: user.roles,
        })
    }

    pub async fn logout(&self, refresh_token: &str) -> Result<()> {
        // Parse token without validation to get the JTI
        let claims = match validate_refresh_token(refresh_token, "dummy_secret_for_parsing") {
            Ok(claims) => claims,
            Err(JwtError::InvalidSignature) => {
                // Extract claims manually for JTI
                let parts: Vec<&str> = refresh_token.split('.').collect();
                if parts.len() != 3 {
                    return Err(Error::Authentication("Invalid token format".to_string()));
                }
                
                let payload = base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
                    .map_err(|_| Error::Authentication("Invalid token payload".to_string()))?;
                
                let claims: crate::jwt::RefreshClaims = serde_json::from_slice(&payload)
                    .map_err(|_| Error::Authentication("Invalid token claims".to_string()))?;
                
                claims
            }
            Err(_) => {
                // If token is invalid, just return success (nothing to logout)
                return Ok(());
            }
        };

        // Invalidate token in Redis
        self.token_repo.invalidate_refresh_token(&claims.jti).await?;

        Ok(())
    }
}
