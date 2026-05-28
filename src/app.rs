use crate::config::AppConfig;
use crate::middleware::auth::{jwt_middleware, require_active_account, require_rv_terminal};
use crate::routes::{auth, statistics, user};
use crate::state::AppState;

use axum::{Router, middleware};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

pub async fn build_router(state: AppState) -> Router {
    let public = Router::new()
        .nest("/api/v2/authenticate", auth::v2::routes())
        .nest("/api/v1/authenticate", auth::v1::routes())
        .nest("/api/v1/user", user::public_routes());

    let rv_terminal_protected = Router::new()
        .nest("/api/v1/user", user::protected_routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_rv_terminal,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_active_account,
        ));

    let rv_terminal_public = Router::new()
        .nest("/api/v1/statistics", statistics::routes())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_rv_terminal,
        ));

    Router::new()
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
