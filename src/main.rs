mod app;
pub mod config;
pub mod db;
mod error;
pub mod middleware;
mod routes;
mod state;

#[tokio::main]
async fn main() {
    let (app, listener) = app::create_app().await;
    app::serve(app, listener).await;
}
