use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;

use crate::{
    db::{self, history::PurchaseHistoryData},
    error::Result,
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Serialize)]
pub struct PurchaseHistoryResponse {
    pub purchases: Vec<PurchaseHistoryData>,
}

#[derive(Serialize)]
pub struct PurchaseHistoryRecordResponse {
    pub purchase: PurchaseHistoryData,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(find_users_purchase_history))
        .route("/{category_id}", get(find_purchase_history_record))
}

async fn find_users_purchase_history(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::history::find_purchase_history_by_user_id(auth.user_id, &state.database).await {
        Ok(purchases) => {
            tracing::info!("User with id {} fetched categories", auth.user_id);
            Ok(Json(PurchaseHistoryResponse { purchases }))
        }
        Err(e) => Err(e),
    }
}

async fn find_purchase_history_record(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(purchase_id): Path<i32>,
) -> Result<Json<PurchaseHistoryRecordResponse>> {
    let purchase = db::history::find_purchase_history_by_user_id_and_purchase_id(
        auth.user_id,
        purchase_id,
        &state.database,
    )
    .await?;

    tracing::info!(
        "User with id {} queried purchase with id {}",
        auth.user_id,
        purchase_id
    );
    Ok(Json(PurchaseHistoryRecordResponse { purchase }))
}
