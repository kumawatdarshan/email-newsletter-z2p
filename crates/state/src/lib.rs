use axum::extract::FromRef;
use email_client::EmailClient;
use settings::Configuration;
use sqlx::SqlitePool;

/// State needed for various services like ~psql~,sqlite, redis, etc
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub email_client: EmailClient,
    pub base_url: String,
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
