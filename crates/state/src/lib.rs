use std::ops::Deref;

use axum::extract::FromRef;
use email_client::EmailClient;
use secrecy::SecretString;
use settings::Configuration;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct HmacSecret(pub SecretString);

impl Deref for HmacSecret {
    type Target = SecretString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<SecretString> for HmacSecret {
    fn from(value: SecretString) -> Self {
        Self(value)
    }
}

/// State needed for various services like ~psql~,sqlite, redis, etc
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub email_client: EmailClient,
    pub base_url: String,
    pub hmac_secret: HmacSecret,
}

pub fn create_email_client(config: &Configuration) -> EmailClient {
    let sender_email = config.email_client.sender().expect("Invalid Sender email");
    let timeout = config.email_client.timeout();

    EmailClient::new(
        config.email_client.base_url.clone(),
        sender_email,
        config.email_client.authorization_token.clone(),
        timeout,
    )
}
