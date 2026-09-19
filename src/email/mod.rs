use crate::{config::AppConfig, db::user::User, error::Result};
use lettre::{
    Message, SmtpTransport, Transport, message::MultiPart, transport::smtp::response::Response,
};
use std::{fs, path::Path, time::Duration};

pub fn send_temp_password(
    user: &User,
    temp_password: &str,
    mailer: &SmtpTransport,
) -> Result<Response> {
    let first_name = user
        .full_name
        .split_whitespace()
        .next()
        .unwrap_or("No name");

    let data = serde_json::json!({
        "name": first_name,
        "username": user.username,
        "temp_password": temp_password
    });

    let text_template_path = Path::new("templates/temp_password.txt");
    let html_template_path = Path::new("templates/temp_password.html");

    let text_body = render_template(text_template_path, &data)?;
    let html_body = render_template(html_template_path, &data)?;

    let email = Message::builder()
        .from("TKO-äly RV <noreply@tko-aly.fi>".parse()?)
        .to(user.email.parse()?)
        .subject("Temporary RV password")
        .multipart(MultiPart::alternative_plain_html(text_body, html_body))?;

    let response = mailer.send(&email)?;

    return Ok(response);
}

fn render_template(template_path: &Path, data: &serde_json::Value) -> Result<String> {
    let template = fs::read_to_string(template_path)?;

    let mut env = minijinja::Environment::new();

    env.add_template("email", &template)?;

    let jinja_template = env.get_template("email")?;

    Ok(jinja_template.render(data)?)
}

pub fn create_smtp_connection(config: &AppConfig) -> SmtpTransport {
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

    return mailer;
}
