mod app;
pub mod config;
pub mod db;
pub mod middleware;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    let (app, listener) = app::build_service().await;
    app::serve(app, listener);
}
