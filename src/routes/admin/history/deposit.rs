use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};

use crate::{
    db,
    error::AppError,
    middleware::auth::AuthUser,
    routes::history::deposit::{DepositHistoryRecordResponse, DepositHistoryResponse},
    state::AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(find_deposit_history))
        .route("/{deposit_id}", get(find_deposit_history_event_with_id))
}

async fn find_deposit_history(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::history::find_deposit_history(None, None, &state.database).await {
        Ok(deposits) => {
            tracing::info!(
                "User with id {} queried whole deposit history as admin",
                auth.user_id
            );
            Ok(Json(DepositHistoryResponse { deposits }))
        }
        Err(_) => Err(AppError::Internal(anyhow::format_err!(
            "No one has deposited money",
        ))),
    }
}

async fn find_deposit_history_event_with_id(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(deposit_id): Path<i32>,
) -> impl IntoResponse {
    match db::history::find_deposit_by_id(deposit_id, &state.database).await {
        Ok(deposit) => {
            tracing::info!(
                "User with id {} fetched deposit {} as admin",
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
