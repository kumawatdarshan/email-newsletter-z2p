use sqlx::PgPool;
use tokio::net::TcpListener;
use z2p::app_state::create_email_client;
use z2p::{
    app_state::init_tracing,
    configuration::{AppState, get_configuration},
    routes::get_router,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_tracing()?;
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
