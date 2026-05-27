pub mod v1;
pub mod v2;

use serde::Deserialize;

#[derive(Deserialize)]
struct LoginRfidRequest {
    rfid: String,
    #[serde(rename = "rvTerminalSecret")]
    rv_terminal_secret: Option<String>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
    #[serde(rename = "rvTerminalSecret")]
    rv_terminal_secret: Option<String>,
    #[serde(rename = "passwordReset")]
    password_reset: bool,
}
