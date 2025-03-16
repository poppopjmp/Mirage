#[macro_use] extern crate rocket;

use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::State;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: Uuid,
    username: String,
    email: String,
    role: String,
    team_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Team {
    id: Uuid,
    name: String,
    members: Vec<Uuid>,
}

type UserDb = Mutex<Vec<User>>;
type TeamDb = Mutex<Vec<Team>>;

#[post("/users", format = "json", data = "<user>")]
fn create_user(user: Json<User>, user_db: &State<UserDb>) -> Json<User> {
    let mut users = user_db.lock().unwrap();
    let new_user = User {
        id: Uuid::new_v4(),
        ..user.into_inner()
    };
    users.push(new_user.clone());
    Json(new_user)
}

#[get("/users/<id>")]
fn get_user(id: String, user_db: &State<UserDb>) -> Option<Json<User>> {
    let users = user_db.lock().unwrap();
    users.iter().find(|&user| user.id.to_string() == id).cloned().map(Json)
}

#[put("/users/<id>", format = "json", data = "<user>")]
fn update_user(id: String, user: Json<User>, user_db: &State<UserDb>) -> Option<Json<User>> {
    let mut users = user_db.lock().unwrap();
    if let Some(existing_user) = users.iter_mut().find(|user| user.id.to_string() == id) {
        *existing_user = user.into_inner();
        Some(Json(existing_user.clone()))
    } else {
        None
    }
}

#[delete("/users/<id>")]
fn delete_user(id: String, user_db: &State<UserDb>) -> Option<Json<User>> {
    let mut users = user_db.lock().unwrap();
    if let Some(pos) = users.iter().position(|user| user.id.to_string() == id) {
        Some(Json(users.remove(pos)))
    } else {
        None
    }
}

#[post("/teams", format = "json", data = "<team>")]
fn create_team(team: Json<Team>, team_db: &State<TeamDb>) -> Json<Team> {
    let mut teams = team_db.lock().unwrap();
    let new_team = Team {
        id: Uuid::new_v4(),
        ..team.into_inner()
    };
    teams.push(new_team.clone());
    Json(new_team)
}

#[get("/teams/<id>")]
fn get_team(id: String, team_db: &State<TeamDb>) -> Option<Json<Team>> {
    let teams = team_db.lock().unwrap();
    teams.iter().find(|&team| team.id.to_string() == id).cloned().map(Json)
}

#[put("/teams/<id>", format = "json", data = "<team>")]
fn update_team(id: String, team: Json<Team>, team_db: &State<TeamDb>) -> Option<Json<Team>> {
    let mut teams = team_db.lock().unwrap();
    if let Some(existing_team) = teams.iter_mut().find(|team| team.id.to_string() == id) {
        *existing_team = team.into_inner();
        Some(Json(existing_team.clone()))
    } else {
        None
    }
}

#[delete("/teams/<id>")]
fn delete_team(id: String, team_db: &State<TeamDb>) -> Option<Json<Team>> {
    let mut teams = team_db.lock().unwrap();
    if let Some(pos) = teams.iter().position(|team| team.id.to_string() == id) {
        Some(Json(teams.remove(pos)))
    } else {
        None
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Mutex::new(Vec::<User>::new()))
        .manage(Mutex::new(Vec::<Team>::new()))
        .mount("/", routes![
            create_user,
            get_user,
            update_user,
            delete_user,
            create_team,
            get_team,
            update_team,
            delete_team,
        ])
}
