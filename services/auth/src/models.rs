use chrono::{DateTime, Utc};
use mirage_common::models::User;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserModel> for User {
    fn from(model: UserModel) -> Self {
        User {
            id: model.id,
            username: model.username,
            email: model.email,
            password_hash: Some(model.password_hash),
            roles: model.roles,
            is_active: model.is_active,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LoginResult {
    pub user_id: Uuid,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RefreshResult {
    pub user_id: Uuid,
    pub roles: Vec<String>,
}
