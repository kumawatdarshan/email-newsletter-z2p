use routes::get_router;
use server::create_email_client;
use settings::{AppState, get_configuration};
use sqlx::PgPool;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    telemetry::init_tracing()?;
    let config = get_configuration().expect("Failed to read Configuration");

    let base_url = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(&base_url).await?;

    let pool = PgPool::connect_lazy_with(config.database.with_db());

    let email_client = create_email_client(&config);

    let app_state = AppState {
        db_pool: pool,
        email_client,
        base_url,
    };

    let router = get_router(app_state);

    axum::serve(listener, router).await
}
