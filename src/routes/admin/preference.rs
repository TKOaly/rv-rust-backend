use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use serde::Serialize;

use crate::{
    db::{self, preference::Preference},
    error::{AppError, Result},
    middleware::auth::AuthUser,
    state::AppState,
};

#[derive(Serialize)]
struct PreferenceResponse {
    preference: Preference,
}

#[derive(Serialize)]
struct PreferencesResponse {
    preferences: Vec<Preference>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_preferences))
        .route("/{key}", get(get_preference_by_key))
}

async fn get_preferences(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
) -> impl IntoResponse {
    match db::preference::get_all_preferences(&state.database).await {
        Ok(preferences) => {
            tracing::info!(
                "User with id {} fetched all preferences as admin",
                auth.user_id
            );
            Ok(Json(PreferencesResponse { preferences }))
        }
        Err(e) => Err(e),
    }
}

async fn get_preference_by_key(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthUser>,
    Path(key): Path<String>,
) -> Result<Json<PreferenceResponse>> {
    let preference = db::preference::find_preference_by_key(&key, &state.database)
        .await?
        .ok_or_else(|| AppError::NotFound("Preference does not exist".to_string()))?;

    tracing::info!(
        "User with id {} fetched preference by key {} as admin",
        auth.user_id,
        key
    );
    Ok(Json(PreferenceResponse { preference }))
}
