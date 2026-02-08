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

pub async fn get_redis(host: String, port: u16) -> anyhow::Result<tower_sessions_redis_store::fred::prelude::Pool> {
    use tower_sessions_redis_store::fred::{
        clients::Pool, interfaces::ClientLike, prelude::ServerConfig, types::config::Config,
    };

    let config = Config {
        server: ServerConfig::new_centralized(host, port),
        ..Config::default()
    };

    let redis_pool = Pool::new(config, None, None, None, 6)?;

    let _redis_join_handle = redis_pool.connect();

    redis_pool.wait_for_connect().await?;

    Ok(redis_pool)
}
