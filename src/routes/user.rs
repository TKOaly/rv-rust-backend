use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        self,
        user::{UpdateUser, User},
    },
    error::AppError,
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
    pub privacy_level: i32,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            user_id: user.id,
            username: user.username,
            full_name: user.full_name,
            email: user.email,
            money_balance: user.saldo,
            role: format!("{:?}", user.role),
            privacy_level: user.privacy_level.into(),
        }
    }
}

#[derive(Deserialize)]
struct UserUpdateRequest {
    username: Option<String>,
    #[serde(rename = "fullName")]
    full_name: Option<String>,
    email: Option<String>,
}

impl From<UserUpdateRequest> for UpdateUser {
    fn from(user: UserUpdateRequest) -> Self {
        Self {
            username: user.username,
            full_name: user.full_name,
            email: user.email,
            role: None,
            saldo: None,
            privacy_level: None,
            password: None,
            rfid: None,
        }
    }
}

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/user_exists", post(user_exists))
}

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/user", get(get_user))
        .route("/", patch(update_user))
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

async fn update_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    body: Json<UserUpdateRequest>,
) -> impl IntoResponse {
    if let Some(username) = &body.username {
        if let Ok(user) = db::user::find_user_by_username(username, &state.database).await {
            tracing::warn!(
                "User {} tried to change username to {} but it was taken",
                user.username,
                username
            );
            return AppError::IdentifierTaken("Username already in use.".to_string())
                .into_response();
        };
    }

    if let Some(email) = &body.email {
        if let Ok(user) = db::user::find_user_by_email(email, &state.database).await {
            tracing::warn!(
                "User {} tried to change email from {} to {} but it was taken",
                user.username,
                user.email,
                email,
            );
            return AppError::IdentifierTaken("Email address already in use.".to_string())
                .into_response();
        }
    }

    let response =
        match db::user::update_user(auth.user_id, body.0.into(), &state.config, &state.database)
            .await
        {
            Ok(u) => {
                tracing::info!(
                    "User {:?} changed user data ({:?}, {:?}, {:?}). None values not updated.",
                    u.username,
                    u.username,
                    u.full_name,
                    u.email
                );

                (StatusCode::OK, Json(UserResponse::from(u))).into_response()
            }
            Err(e) => e.into_response(),
        };

    response
}
