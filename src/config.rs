#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub database_url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4040),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_owned()),
            database_url: std::env::var("DATABASE_URL").expect("database url must be set"),
        }
    }
}
