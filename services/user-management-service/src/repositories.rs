use crate::config::DatabaseConfig;
use crate::models::{RoleModel, TeamMemberModel, TeamModel, UserModel};
use mirage_common::{Error, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
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

/// User repository for database operations
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<UserModel>> {
        let users = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            ORDER BY username
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch users: {}", e)))?;

        Ok(users)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<UserModel>> {
        let user = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find user: {}", e)))?;

        Ok(user)
    }

    pub async fn create(&self, user: &UserModel) -> Result<UserModel> {
        let created = sqlx::query_as!(
            UserModel,
            r#"
            INSERT INTO users (id, username, email, roles, is_active)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, username, email, roles as "roles: Vec<String>", is_active, created_at, updated_at
            "#,
            user.id,
            user.username,
            user.email,
            &user.roles as _,
            user.is_active
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create user: {}", e)))?;

        Ok(created)
    }

    pub async fn update(&self, user: &UserModel) -> Result<UserModel> {
        let updated = sqlx::query_as!(
            UserModel,
            r#"
            UPDATE users
            SET username = $2, email = $3, roles = $4, is_active = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING id, username, email, roles as "roles: Vec<String>", is_active, created_at, updated_at
            "#,
            user.id,
            user.username,
            user.email,
            &user.roles as _,
            user.is_active
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update user: {}", e)))?;

        Ok(updated)
    }

    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete user: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<UserModel>> {
        let user = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find user by username: {}", e)))?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<UserModel>> {
        let user = sqlx::query_as!(
            UserModel,
            r#"
            SELECT id, username, email, roles as "roles: Vec<String>", is_active, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find user by email: {}", e)))?;

        Ok(user)
    }
}

/// Team repository for database operations
pub struct TeamRepository {
    pool: DbPool,
}

impl TeamRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, team: &TeamModel) -> Result<TeamModel> {
        let created = sqlx::query_as!(
            TeamModel,
            r#"
            INSERT INTO teams (id, name, description)
            VALUES ($1, $2, $3)
            RETURNING id, name, description, created_at, updated_at
            "#,
            team.id,
            team.name,
            team.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create team: {}", e)))?;

        Ok(created)
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<TeamModel>> {
        let teams = sqlx::query_as!(
            TeamModel,
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM teams
            ORDER BY name
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch teams: {}", e)))?;

        Ok(teams)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<TeamModel>> {
        let team = sqlx::query_as!(
            TeamModel,
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM teams
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find team: {}", e)))?;

        Ok(team)
    }

    pub async fn update(&self, team: &TeamModel) -> Result<TeamModel> {
        let updated = sqlx::query_as!(
            TeamModel,
            r#"
            UPDATE teams
            SET name = $2, description = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, description, created_at, updated_at
            "#,
            team.id,
            team.name,
            team.description
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update team: {}", e)))?;

        Ok(updated)
    }

    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM teams
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete team: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_member(&self, member: &TeamMemberModel) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO team_members (team_id, user_id, role, joined_at)
            VALUES ($1, $2, $3, $4)
            "#,
            member.team_id,
            member.user_id,
            member.role,
            member.joined_at
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to add team member: {}", e)))?;

        Ok(())
    }

    pub async fn remove_member(&self, team_id: &Uuid, user_id: &Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM team_members
            WHERE team_id = $1 AND user_id = $2
            "#,
            team_id,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to remove team member: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn get_team_members(&self, team_id: &Uuid) -> Result<Vec<TeamMemberModel>> {
        let members = sqlx::query_as!(
            TeamMemberModel,
            r#"
            SELECT team_id, user_id, role, joined_at
            FROM team_members
            WHERE team_id = $1
            "#,
            team_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to get team members: {}", e)))?;

        Ok(members)
    }

    pub async fn get_user_teams(&self, user_id: &Uuid) -> Result<Vec<(TeamModel, String)>> {
        // This is a more complex query that joins teams with team_members
        // We'd ideally use a custom return type, but for simplicity, we'll return a tuple
        let teams = sqlx::query!(
            r#"
            SELECT t.id, t.name, t.description, t.created_at, t.updated_at, tm.role
            FROM teams t
            JOIN team_members tm ON t.id = tm.team_id
            WHERE tm.user_id = $1
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to get user teams: {}", e)))?;

        let result = teams
            .into_iter()
            .map(|row| {
                (
                    TeamModel {
                        id: row.id,
                        name: row.name,
                        description: row.description,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                    },
                    row.role,
                )
            })
            .collect();

        Ok(result)
    }
}

/// Role repository for database operations
pub struct RoleRepository {
    pool: DbPool,
}

impl RoleRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, role: &RoleModel) -> Result<RoleModel> {
        let created = sqlx::query_as!(
            RoleModel,
            r#"
            INSERT INTO roles (id, name, description, permissions)
            VALUES ($1, $2, $3, $4)
            RETURNING id, name, description, permissions as "permissions: Vec<String>", created_at, updated_at
            "#,
            role.id,
            role.name,
            role.description,
            &role.permissions as _
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create role: {}", e)))?;

        Ok(created)
    }

    pub async fn find_all(&self) -> Result<Vec<RoleModel>> {
        let roles = sqlx::query_as!(
            RoleModel,
            r#"
            SELECT id, name, description, permissions as "permissions: Vec<String>", created_at, updated_at
            FROM roles
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch roles: {}", e)))?;

        Ok(roles)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<RoleModel>> {
        let role = sqlx::query_as!(
            RoleModel,
            r#"
            SELECT id, name, description, permissions as "permissions: Vec<String>", created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find role: {}", e)))?;

        Ok(role)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<RoleModel>> {
        let role = sqlx::query_as!(
            RoleModel,
            r#"
            SELECT id, name, description, permissions as "permissions: Vec<String>", created_at, updated_at
            FROM roles
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find role by name: {}", e)))?;

        Ok(role)
    }

    pub async fn update(&self, role: &RoleModel) -> Result<RoleModel> {
        let updated = sqlx::query_as!(
            RoleModel,
            r#"
            UPDATE roles
            SET name = $2, description = $3, permissions = $4, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, description, permissions as "permissions: Vec<String>", created_at, updated_at
            "#,
            role.id,
            role.name,
            role.description,
            &role.permissions as _
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update role: {}", e)))?;

        Ok(updated)
    }

    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM roles
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete role: {}", e)))?;

        Ok(result.rows_affected() > 0)
    }
}
