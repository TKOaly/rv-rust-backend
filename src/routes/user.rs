use axum::{
    Extension, Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, user::User},
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Deserialize)]
struct UserExistsRequest {
    username: String,
}

#[derive(Serialize)]
struct UserExistsResponse {
    exists: bool,
}

#[derive(Deserialize, Serialize)]
struct UserResponse {
    #[serde(rename = "userId")]
    pub user_id: i32,
    pub username: String,
    #[serde(rename = "fullName")]
    pub full_name: String,
    #[serde(rename = "email")]
    pub email: String,
    #[serde(rename = "moneyBalance")]
    pub money_balance: i32,
    pub role: String,
    #[serde(rename = "privacyLevel")]
    pub privacy_level: u8,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
            full_name: user.realname,
            email: user.email,
            money_balance: user.saldo,
            role: format!("{:?}", user.role),
            privacy_level: user.privacy_level.into(),
        }
    }
}

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/user_exists", post(user_exists))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new().route("/user", get(get_user))
}

async fn user_exists(
    State(state): State<AppState>,
    body: Json<UserExistsRequest>,
) -> Json<UserExistsResponse> {
    match db::user::find_user_by_username(&body.username, &state.database).await {
        Ok(_) => Json(UserExistsResponse { exists: true }),
        Err(_) => Json(UserExistsResponse { exists: false }),
    }
}

async fn get_user(State(state): State<AppState>, body: Json<UserResponse>) -> impl IntoResponse {
    let user = match db::user::find_user_by_id(body.user_id, &state.database).await {
        Ok(u) => u,
        Err(e) => return Err(e),
    };

    tracing::info!("User {} fetched user data", user.username);
    Ok(Json(UserResponse::from(user)))
}
