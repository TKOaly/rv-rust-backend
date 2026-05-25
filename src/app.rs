use crate::config::AppConfig;
use crate::state::AppState;
use axum::Router;
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt};

pub async fn build_router(state: AppState) -> Router {
    Router::new().with_state(state)
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
