use crate::config::AppConfig;
use lettre::SmtpTransport;
use sea_orm::{Database, DatabaseConnection};
use std::{sync::Arc, time::Duration};

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

fn create_smtp_connection(config: &AppConfig) -> SmtpTransport {
    tracing::info!(
        "Trying to establish a TLS wrapped connection to {}",
        config.smtp_host
    );

    let mailer = SmtpTransport::relay(&config.smtp_host)
        .expect("build SmtpTransport::relay")
        .timeout(Some(Duration::from_secs(10)))
        .build();

    match mailer.test_connection() {
        Ok(true) => {
            tracing::info!(
                "Successfully connected to {} via a TLS wrapped connection (SmtpTransport::relay). This is the fastest option available for connecting to an SMTP server",
                config.smtp_host
            );
        }
        Ok(false) => {
            tracing::error!(
                "Couldn't connect to {} via a TLS wrapped connection. No more information is available",
                config.smtp_host
            );
        }
        Err(err) => {
            tracing::error!(err = %err, "Couldn't connect to {} via a TLS wrapped connection", config.smtp_host);
        }
    }

    return mailer
}
