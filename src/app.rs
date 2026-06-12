use crate::config::AppConfig;
use crate::db::user::Role;
use crate::middleware::auth::{
    jwt_middleware, require_active_account, require_role, require_rv_terminal,
};
use crate::routes::{admin, auth, category, product, register, statistics, user};
use crate::state::AppState;

use axum::{Router, middleware};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

pub async fn build_router(state: AppState) -> Router {
    let rv_terminal_admin = Router::new()
        .nest("/api/v1/categories", admin::category::routes())
        .nest("/api/v1/preferences", admin::preference::routes())
        .nest("/api/v1/utils", admin::utils::routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            |state, req, next| require_role(state, Role::Admin, req, next),
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_rv_terminal,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_middleware,
        ));

    let rv_terminal_protected = Router::new()
        .nest("/api/v1/user", user::protected_routes())
        .nest("/api/v1/products", product::routes())
        .nest("/api/v1/categories", category::routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_active_account,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_rv_terminal,
        ))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_middleware,
        ));

    let rv_terminal_public = Router::new()
        .nest("/api/v1/statistics", statistics::routes())
        .nest("/api/v1/register", register::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_rv_terminal,
        ));

    let public = Router::new()
        .nest("/api/v2/authenticate", auth::v2::routes())
        .nest("/api/v1/authenticate", auth::v1::routes())
        .nest("/api/v1/user", user::public_routes());

    Router::new()
        .merge(rv_terminal_admin)
        .merge(rv_terminal_protected)
        .merge(rv_terminal_public)
        .merge(public)
        .with_state(state)
}

pub async fn create_app() -> (Router, TcpListener) {
    dotenv::dotenv().ok();

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json() // comment out to get text output
        .with_span_events(fmt::format::FmtSpan::FULL) // optional: include enter/exit span events
        .init();

    let config = AppConfig::from_env();
    let state = AppState::new(&config).await;

    let router = build_router(state).await;

    let adders = format!("{}:{}", config.host, config.port);

    let listener = TcpListener::bind(adders)
        .await
        .map_err(|e| {
            eprintln!("unable to parse local address: {e}");
        })
        .unwrap();

    (router, listener)
}

pub async fn serve(app: Router, listener: TcpListener) {
    axum::serve(listener, app).await.unwrap();
}
