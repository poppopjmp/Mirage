use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, errors, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // Subject (user_id)
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub roles: Vec<String>, // User roles
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: String,        // Subject (user_id)
    pub exp: i64,           // Expiration time
    pub iat: i64,           // Issued at
    pub jti: String,        // JWT ID
}

/// Generate JWT access token
pub fn generate_token(
    user_id: &Uuid,
    roles: &[String],
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiration_seconds)).timestamp();
    
    let claims = Claims {
        sub: user_id.to_string(),
        exp,
        iat: now.timestamp(),
        roles: roles.to_vec(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Generate refresh token
pub fn generate_refresh_token(
    user_id: &Uuid,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiration_seconds)).timestamp();
    
    let claims = RefreshClaims {
        sub: user_id.to_string(),
        exp,
        iat: now.timestamp(),
        jti: Uuid::new_v4().to_string(),
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Validate access token and extract claims
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    
    Ok(token_data.claims)
}

/// Validate refresh token and extract claims
pub fn validate_refresh_token(token: &str, secret: &str) -> Result<RefreshClaims, errors::Error> {
    let token_data = decode::<RefreshClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )?;
    
    Ok(token_data.claims)
}
