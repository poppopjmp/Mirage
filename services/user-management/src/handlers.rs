use crate::models::{CreateUserRequest, UpdateUserRequest, CreateTeamRequest, UpdateTeamRequest, TeamModel, RoleModel};
use crate::services::UserService;
use mirage_common::{Error, models::User};
use rocket::http::Status;
use rocket::serde::json::{Json, Value, json};
use rocket::State;
use uuid::Uuid;

// Helper to format errors consistently
fn error_response(error: mirage_common::Error) -> (Status, Value) {
    let status = match &error {
        Error::NotFound(_) => Status::NotFound,
        Error::Validation(_) => Status::BadRequest,
        Error::Authentication(_) => Status::Unauthorized,
        Error::Authorization(_) => Status::Forbidden,
        _ => Status::InternalServerError,
    };

    (status, json!({ "error": error.to_string() }))
}

// User Routes
#[get("/")]
async fn get_users(service: &State<UserService>) -> Result<Json<Vec<User>>, (Status, Value)> {
    service.get_users(100, 0).await
        .map(Json)
        .map_err(error_response)
}

#[get("/<id>")]
async fn get_user(id: &str, service: &State<UserService>) -> Result<Json<User>, (Status, Value)> {
    let user_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid user ID format".to_string()))
    )?;
    
    service.get_user(&user_id).await
        .map(Json)
        .map_err(error_response)
}

#[post("/", data = "<user>")]
async fn create_user(user: Json<CreateUserRequest>, service: &State<UserService>) -> Result<Json<User>, (Status, Value)> {
    service.create_user(user.into_inner()).await
        .map(Json)
        .map_err(error_response)
}

#[put("/<id>", data = "<user>")]
async fn update_user(id: &str, user: Json<UpdateUserRequest>, service: &State<UserService>) -> Result<Json<User>, (Status, Value)> {
    let user_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid user ID format".to_string()))
    )?;
    
    service.update_user(&user_id, user.into_inner()).await
        .map(Json)
        .map_err(error_response)
}

#[delete("/<id>")]
async fn delete_user(id: &str, service: &State<UserService>) -> Result<Status, (Status, Value)> {
    let user_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid user ID format".to_string()))
    )?;
    
    service.delete_user(&user_id).await
        .map(|deleted| if deleted { Status::NoContent } else { Status::NotFound })
        .map_err(error_response)
}

// Team Routes
#[get("/")]
async fn get_teams(service: &State<UserService>) -> Result<Json<Vec<TeamModel>>, (Status, Value)> {
    service.get_teams(100, 0).await
        .map(Json)
        .map_err(error_response)
}

#[get("/<id>")]
async fn get_team(id: &str, service: &State<UserService>) -> Result<Json<TeamModel>, (Status, Value)> {
    let team_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid team ID format".to_string()))
    )?;
    
    service.get_team(&team_id).await
        .map(Json)
        .map_err(error_response)
}

#[post("/", data = "<team>")]
async fn create_team(team: Json<CreateTeamRequest>, service: &State<UserService>) -> Result<Json<TeamModel>, (Status, Value)> {
    service.create_team(team.into_inner()).await
        .map(Json)
        .map_err(error_response)
}

#[put("/<id>", data = "<team>")]
async fn update_team(id: &str, team: Json<UpdateTeamRequest>, service: &State<UserService>) -> Result<Json<TeamModel>, (Status, Value)> {
    let team_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid team ID format".to_string()))
    )?;
    
    service.update_team(&team_id, team.into_inner()).await
        .map(Json)
        .map_err(error_response)
}

#[delete("/<id>")]
async fn delete_team(id: &str, service: &State<UserService>) -> Result<Status, (Status, Value)> {
    let team_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid team ID format".to_string()))
    )?;
    
    service.delete_team(&team_id).await
        .map(|deleted| if deleted { Status::NoContent } else { Status::NotFound })
        .map_err(error_response)
}

#[get("/<id>/members")]
async fn get_team_members(id: &str, service: &State<UserService>) -> Result<Json<Vec<(User, String)>>, (Status, Value)> {
    let team_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid team ID format".to_string()))
    )?;
    
    service.get_team_members(&team_id).await
        .map(Json)
        .map_err(error_response)
}

#[derive(serde::Deserialize)]
struct TeamMemberRequest {
    role: String,
}

#[put("/<team_id>/members/<user_id>", data = "<data>")]
async fn add_team_member(team_id: &str, user_id: &str, data: Json<TeamMemberRequest>, service: &State<UserService>) -> Result<Status, (Status, Value)> {
    let team_id = Uuid::parse_str(team_id).map_err(|_| 
        error_response(Error::Validation("Invalid team ID format".to_string()))
    )?;
    
    let user_id = Uuid::parse_str(user_id).map_err(|_| 
        error_response(Error::Validation("Invalid user ID format".to_string()))
    )?;
    
    service.add_team_member(&team_id, &user_id, &data.role).await
        .map(|_| Status::Created)
        .map_err(error_response)
}

#[delete("/<team_id>/members/<user_id>")]
async fn remove_team_member(team_id: &str, user_id: &str, service: &State<UserService>) -> Result<Status, (Status, Value)> {
    let team_id = Uuid::parse_str(team_id).map_err(|_| 
        error_response(Error::Validation("Invalid team ID format".to_string()))
    )?;
    
    let user_id = Uuid::parse_str(user_id).map_err(|_| 
        error_response(Error::Validation("Invalid user ID format".to_string()))
    )?;
    
    service.remove_team_member(&team_id, &user_id).await
        .map(|removed| if removed { Status::NoContent } else { Status::NotFound })
        .map_err(error_response)
}

// Role Routes
#[get("/")]
async fn get_roles(service: &State<UserService>) -> Result<Json<Vec<RoleModel>>, (Status, Value)> {
    service.get_roles().await
        .map(Json)
        .map_err(error_response)
}

#[get("/<id>")]
async fn get_role(id: &str, service: &State<UserService>) -> Result<Json<RoleModel>, (Status, Value)> {
    let role_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid role ID format".to_string()))
    )?;
    
    service.get_role(&role_id).await
        .map(Json)
        .map_err(error_response)
}

#[post("/", data = "<role>")]
async fn create_role(role: Json<RoleModel>, service: &State<UserService>) -> Result<Json<RoleModel>, (Status, Value)> {
    service.create_role(role.into_inner()).await
        .map(Json)
        .map_err(error_response)
}

#[put("/<id>", data = "<role>")]
async fn update_role(id: &str, mut role: Json<RoleModel>, service: &State<UserService>) -> Result<Json<RoleModel>, (Status, Value)> {
    let role_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid role ID format".to_string()))
    )?;
    
    // Ensure the ID in the path matches the ID in the body
    if role.id != role_id {
        return Err(error_response(Error::Validation("Role ID in URL must match ID in body".to_string())));
    }
    
    service.update_role(role.into_inner()).await
        .map(Json)
        .map_err(error_response)
}

#[delete("/<id>")]
async fn delete_role(id: &str, service: &State<UserService>) -> Result<Status, (Status, Value)> {
    let role_id = Uuid::parse_str(id).map_err(|_| 
        error_response(Error::Validation("Invalid role ID format".to_string()))
    )?;
    
    service.delete_role(&role_id).await
        .map(|deleted| if deleted { Status::NoContent } else { Status::NotFound })
        .map_err(error_response)
}

// Route collections
pub fn user_routes() -> Vec<rocket::Route> {
    routes![
        get_users,
        get_user,
        create_user,
        update_user,
        delete_user
    ]
}

pub fn team_routes() -> Vec<rocket::Route> {
    routes![
        get_teams,
        get_team,
        create_team,
        update_team,
        delete_team,
        get_team_members,
        add_team_member,
        remove_team_member
    ]
}

pub fn role_routes() -> Vec<rocket::Route> {
    routes![
        get_roles,
        get_role,
        create_role,
        update_role,
        delete_role
    ]
}
