use anyhow::{Context, Result};
use routes::get_router;
use settings::get_configuration;
use sqlx::SqlitePool;
use state::{AppState, create_email_client};
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

    let email_client = create_email_client(&config);

    let app_state = AppState {
        db_pool,
        email_client,
        base_url,
        hmac_secret: config.application.hmac_secret.into(),
    };

    let router = get_router(app_state);

    axum::serve(listener, router)
        .await
        .context("Failed to serve application using axum")?;

    Ok(())
}
