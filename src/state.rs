use crate::config::AppConfig;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub database: DatabaseConnection,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Self {
        let database = Database::connect(&config.database_url).await.unwrap();

        Self {
            config: Arc::new(config.clone()),
            database,
        }
    }
}
