use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        self,
        user::{UpdateUserData, User},
    },
    error::AppError,
    middleware::auth::AuthUser,
    routes::{
        history::{
            deposit::{DepositHistoryRecordResponse, DepositHistoryResponse},
            purchase::{PurchaseHistoryRecordResponse, PurchaseHistoryResponse},
        },
        user::{PasswordChangeRequest, UserResponse},
    },
    state::AppState,
};

#[derive(Serialize, Deserialize)]
struct RoleChangeRequest {
    role: String,
}

impl From<RoleChangeRequest> for UpdateUserData {
    fn from(request: RoleChangeRequest) -> Self {
        Self {
            username: None,
            full_name: None,
            email: None,
            role: Some(request.role.into()),
            saldo: None,
            privacy_level: None,
            password: None,
            rfid: None,
        }
    }
}

#[derive(Serialize)]
struct UsersResponse {
    users: Vec<UserResponse>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_user))
        .route(
            "/{user_id}/purchaseHistory",
            get(find_users_purchase_history),
        )
        .route(
            "/{user_id}/purchaseHistory/{purchase_id}",
            get(find_users_purchase_history_with_id),
        )
        .route("/{user_id}/depositHistory", get(find_users_deposit_history))
        .route(
            "/{user_id}/depositHistory/{deposit_id}",
            get(find_users_deposit_history_with_id),
        )
        .route("/{user_id}/changePassword", post(change_user_password))
        .route("/{user_id}/changeRole", post(change_user_role))
}

async fn get_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    let users = match db::user::get_all_users(&state.database).await {
        Ok(u) => u,
        Err(e) => return Err(e),
    };

    tracing::info!("User whit id {} fetched users as admin", auth.user_id);

    Ok(Json(UsersResponse {
        users: users
            .iter()
            .map(|u: &User| UserResponse::from(u.to_owned()))
            .collect(),
    }))
}

async fn get_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    let user = match db::user::find_user_by_id(user_id, &state.database).await {
        Ok(u) => u,
        Err(e) => return Err(e),
    };

    tracing::info!(
        "User with id {} fetched users {} data as admin",
        auth.user_id,
        user.username
    );
    Ok(Json(UserResponse::from(user)))
}

async fn find_users_purchase_history(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    match db::history::find_purchase_history_by_user_id(user_id, &state.database).await {
        Ok(purchases) => Ok(Json(PurchaseHistoryResponse { purchases })),
        Err(_) => Err(AppError::NotFound(
            "Purchase user do not have purchases".to_string(),
        )),
    }
}

async fn find_users_purchase_history_with_id(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    Path(purchase_id): Path<i32>,
) -> impl IntoResponse {
    match db::history::find_purchase_history_by_user_id_and_purchase_id(
        user_id,
        purchase_id,
        &state.database,
    )
    .await
    {
        Ok(purchase) => Ok(Json(PurchaseHistoryRecordResponse { purchase })),
        Err(_) => Err(AppError::NotFound(
            "User do not have purchase that purchase".to_string(),
        )),
    }
}

async fn find_users_deposit_history(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
) -> impl IntoResponse {
    match db::history::find_deposit_history_by_user_id(user_id, &state.database).await {
        Ok(deposits) => Ok(Json(DepositHistoryResponse { deposits })),
        Err(_) => Err(AppError::NotFound(
            "User do not have deposited money".to_string(),
        )),
    }
}

async fn find_users_deposit_history_with_id(
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    Path(deposit_id): Path<i32>,
) -> impl IntoResponse {
    match db::history::find_deposit_history_by_user_id_and_deposit_id(
        user_id,
        deposit_id,
        &state.database,
    )
    .await
    {
        Ok(deposit) => Ok(Json(DepositHistoryRecordResponse { deposit })),
        Err(_) => Err(AppError::NotFound(
            "User do not have deposited money".to_string(),
        )),
    }
}

async fn change_user_password(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(user_id): Path<i32>,
    body: Json<PasswordChangeRequest>,
) -> impl IntoResponse {
    let user =
        match db::user::update_user(user_id, body.0.into(), &state.config, &state.database).await {
            Ok(u) => u,
            Err(e) => return e.into_response(),
        };

    tracing::info!(
        "User with id {} changed user's {} password as admin",
        auth.user_id,
        user.username
    );

    StatusCode::NO_CONTENT.into_response()
}

async fn change_user_role(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(user_id): Path<i32>,
    body: Json<RoleChangeRequest>,
) -> impl IntoResponse {
    let user =
        match db::user::update_user(user_id, body.0.into(), &state.config, &state.database).await {
            Ok(u) => u,
            Err(e) => return Err(e),
        };

    tracing::info!(
        "User with id {} changed user's {} role to {:?} as admin",
        auth.user_id,
        user.username,
        user.role
    );

    Ok(Json(RoleChangeRequest {
        role: format!("{:?}", user.role),
    }))
}
