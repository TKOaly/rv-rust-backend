use crate::config::AppConfig;
use crate::email::create_smtp_connection;
use lettre::SmtpTransport;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub database: DatabaseConnection,
    pub mailer: SmtpTransport,
}

impl AppState {
    pub async fn new(config: &AppConfig) -> Self {
        let database = Database::connect(&config.database_url).await.unwrap();
        let mailer = create_smtp_connection(config);

        Self {
            config: Arc::new(config.clone()),
            database,
            mailer,
        }
    }
}
