use crate::config::AppConfig;
use crate::state::AppState;
use axum::Router;
use tokio::net::TcpListener;

pub async fn build_router(state: AppState) -> Router {
    Router::new().with_state(state)
}

pub async fn build_service() -> (Router, TcpListener) {
    dotenv::dotenv().ok();

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
