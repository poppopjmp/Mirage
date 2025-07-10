use crate::models::{
    CreateTeamRequest, CreateUserRequest, RoleModel, TeamMemberModel, TeamModel, UpdateTeamRequest,
    UpdateUserRequest, UserModel,
};
use crate::repositories::{DbPool, RoleRepository, TeamRepository, UserRepository};
use chrono::Utc;
use mirage_common::{models::User, Error, Result};
use uuid::Uuid;

pub struct UserService {
    user_repo: UserRepository,
    team_repo: TeamRepository,
    role_repo: RoleRepository,
}

impl UserService {
    pub fn new(pool: DbPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            team_repo: TeamRepository::new(pool.clone()),
            role_repo: RoleRepository::new(pool),
        }
    }

    // User operations
    pub async fn get_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let users = self.user_repo.find_all(limit, offset).await?;
        Ok(users.into_iter().map(|u| u.into()).collect())
    }

    pub async fn get_user(&self, id: &Uuid) -> Result<User> {
        let user = self
            .user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("User with ID {} not found", id)))?;

        Ok(user.into())
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User> {
        // Validate username and email don't already exist
        if let Some(_) = self.user_repo.find_by_username(&req.username).await? {
            return Err(Error::Validation("Username already taken".to_string()));
        }

        if let Some(_) = self.user_repo.find_by_email(&req.email).await? {
            return Err(Error::Validation("Email already registered".to_string()));
        }

        // Create user model
        let user = UserModel {
            id: Uuid::new_v4(),
            username: req.username,
            email: req.email,
            roles: req.roles,
            is_active: req.is_active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = self.user_repo.create(&user).await?;
        Ok(created.into())
    }

    pub async fn update_user(&self, id: &Uuid, req: UpdateUserRequest) -> Result<User> {
        // Get existing user
        let mut user = self
            .user_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("User with ID {} not found", id)))?;

        // Update fields if provided
        if let Some(username) = req.username {
            // Check if the new username is already taken
            if username != user.username {
                if let Some(_) = self.user_repo.find_by_username(&username).await? {
                    return Err(Error::Validation("Username already taken".to_string()));
                }
            }
            user.username = username;
        }

        if let Some(email) = req.email {
            // Check if the new email is already registered
            if email != user.email {
                if let Some(_) = self.user_repo.find_by_email(&email).await? {
                    return Err(Error::Validation("Email already registered".to_string()));
                }
            }
            user.email = email;
        }

        if let Some(roles) = req.roles {
            user.roles = roles;
        }

        if let Some(is_active) = req.is_active {
            user.is_active = is_active;
        }

        let updated = self.user_repo.update(&user).await?;
        Ok(updated.into())
    }

    pub async fn delete_user(&self, id: &Uuid) -> Result<bool> {
        self.user_repo.delete(id).await
    }

    // Team operations
    pub async fn get_teams(&self, limit: i64, offset: i64) -> Result<Vec<TeamModel>> {
        self.team_repo.find_all(limit, offset).await
    }

    pub async fn get_team(&self, id: &Uuid) -> Result<TeamModel> {
        self.team_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Team with ID {} not found", id)))
    }

    pub async fn create_team(&self, req: CreateTeamRequest) -> Result<TeamModel> {
        let team = TeamModel {
            id: Uuid::new_v4(),
            name: req.name,
            description: req.description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.team_repo.create(&team).await
    }

    pub async fn update_team(&self, id: &Uuid, req: UpdateTeamRequest) -> Result<TeamModel> {
        let mut team = self
            .team_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Team with ID {} not found", id)))?;

        if let Some(name) = req.name {
            team.name = name;
        }

        if let Some(description) = req.description {
            team.description = description;
        }

        self.team_repo.update(&team).await
    }

    pub async fn delete_team(&self, id: &Uuid) -> Result<bool> {
        self.team_repo.delete(id).await
    }

    pub async fn add_team_member(&self, team_id: &Uuid, user_id: &Uuid, role: &str) -> Result<()> {
        // Verify team exists
        self.team_repo
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Team with ID {} not found", team_id)))?;

        // Verify user exists
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("User with ID {} not found", user_id)))?;

        let member = TeamMemberModel {
            team_id: *team_id,
            user_id: *user_id,
            role: role.to_string(),
            joined_at: Utc::now(),
        };

        self.team_repo.add_member(&member).await
    }

    pub async fn remove_team_member(&self, team_id: &Uuid, user_id: &Uuid) -> Result<bool> {
        self.team_repo.remove_member(team_id, user_id).await
    }

    pub async fn get_team_members(&self, team_id: &Uuid) -> Result<Vec<(User, String)>> {
        // Verify team exists
        self.team_repo
            .find_by_id(team_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Team with ID {} not found", team_id)))?;

        let members = self.team_repo.get_team_members(team_id).await?;
        let mut result = Vec::new();

        for member in members {
            if let Some(user) = self.user_repo.find_by_id(&member.user_id).await? {
                result.push((user.into(), member.role));
            }
        }

        Ok(result)
    }

    pub async fn get_user_teams(&self, user_id: &Uuid) -> Result<Vec<(TeamModel, String)>> {
        // Verify user exists
        self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("User with ID {} not found", user_id)))?;

        self.team_repo.get_user_teams(user_id).await
    }

    // Role operations
    pub async fn get_roles(&self) -> Result<Vec<RoleModel>> {
        self.role_repo.find_all().await
    }

    pub async fn get_role(&self, id: &Uuid) -> Result<RoleModel> {
        self.role_repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Role with ID {} not found", id)))
    }

    pub async fn create_role(&self, role: RoleModel) -> Result<RoleModel> {
        self.role_repo.create(&role).await
    }

    pub async fn update_role(&self, role: RoleModel) -> Result<RoleModel> {
        self.role_repo.update(&role).await
    }

    pub async fn delete_role(&self, id: &Uuid) -> Result<bool> {
        self.role_repo.delete(id).await
    }
}
