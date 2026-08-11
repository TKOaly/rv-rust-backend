use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;

use crate::{
    db::{self, history::DepositHistoryData},
    error::AppError,
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Serialize)]
pub struct DepositHistoryResponse {
    pub deposits: Vec<DepositHistoryData>,
}

#[derive(Serialize)]
pub struct DepositHistoryRecordResponse {
    pub deposit: DepositHistoryData,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(find_users_deposit_history))
        .route("/{deposit_id}", get(find_users_deposit_history_with_id))
}

async fn find_users_deposit_history(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::history::find_deposit_history_by_user_id(auth.user_id, &state.database).await {
        Ok(deposits) => {
            tracing::info!("User with id {} deposit history", auth.user_id);
            Ok(Json(DepositHistoryResponse { deposits }))
        }
        Err(_) => Err(AppError::Internal(anyhow::format_err!(
            "User with id {} do not have deposited money",
            auth.user_id
        ))),
    }
}

async fn find_users_deposit_history_with_id(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(deposit_id): Path<i32>,
) -> impl IntoResponse {
    match db::history::find_deposit_history_by_user_id_and_deposit_id(
        auth.user_id,
        deposit_id,
        &state.database,
    )
    .await
    {
        Ok(deposit) => {
            tracing::info!(
                "User with id {} fetched deposit {}",
                auth.user_id,
                deposit_id
            );
            Ok(Json(DepositHistoryRecordResponse { deposit }))
        }
        Err(_) => Err(AppError::NotFound(
            "Deposit event does not exist".to_string(),
        )),
    }
}
