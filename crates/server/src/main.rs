use anyhow::{Context, Result};
use api::{AppState, get_router};
use email_client::EmailClient;
use redis_client::create_redis_pool;
use configuration::get_configuration;
use sqlx::SqlitePool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_tracing().context("Failed to initialize tracing.")?;
    let config = get_configuration().context("Failed to read Configuration.")?;

    let base_url = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(&base_url)
        .await
        .context(format!("Failed to bind to address: {base_url}"))?;

    let db_pool = SqlitePool::connect_lazy_with(config.database.options());

    let email_client = EmailClient::from_config(&config.email_client);

    let app_state = AppState {
        db_pool,
        email_client,
        base_url,
    };

    let redis_pool = create_redis_pool(&config.redis)
        .await
        .context("Failed to initialize redis pool")?;

    let router = get_router(app_state, redis_pool)
        .await
        .expect("Failed to get router");

    axum::serve(listener, router)
        .await
        .context("Failed to serve application using axum")?;

    Ok(())
}
