#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry: i64,
    pub rv_terminal_secret: String,
    pub rfid_salt: [u8; 16],
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(4040),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            database_url: std::env::var("DATABASE_URL").expect("database url must be set"),
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "unsecure".to_string()),
            jwt_expiry: std::env::var("JWT_EXPIRY")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3600),
            rv_terminal_secret: std::env::var("RV_TERMINAL_SECRET")
                .unwrap_or_else(|_| "unsecure".to_string()),
            rfid_salt: std::env::var("RFID_HASH")
                .ok()
                .map(|p| {
                    let mut buf = [0u8; 16];
                    let bytes = p.as_bytes();
                    buf[..bytes.len().min(16)].copy_from_slice(&bytes[..bytes.len().min(16)]);
                    buf
                })
                .unwrap_or([0u8; 16]),
        }
    }
}
