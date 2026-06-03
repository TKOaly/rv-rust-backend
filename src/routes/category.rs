use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;

use crate::{
    db::{self, category::Category},
    error::AppError,
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Serialize)]
struct CategoryResponse {
    category: Category,
}

#[derive(Serialize)]
struct CategoriesResponse {
    categories: Vec<Category>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_categories))
        .route("/{category_id}", get(get_category))
}

async fn get_categories(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::category::get_all_categories(&state.database).await {
        Ok(categories) => {
            tracing::info!("User with id {} fetched categories", auth.user_id);
            Ok(Json(CategoriesResponse { categories }))
        }
        Err(e) => Err(e),
    }
}

async fn get_category(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(category_id): Path<i32>,
) -> Result<Json<CategoryResponse>, AppError> {
    let category = db::category::find_category_by_id(category_id, &state.database)
        .await?
        .ok_or_else(|| AppError::NotFound("Category does not exist".to_string()))?;

    tracing::info!("User with id {} fetched category", auth.user_id);
    Ok(Json(CategoryResponse { category }))
}
