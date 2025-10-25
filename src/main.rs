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

    let bind_addr = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(bind_addr).await?;

    let pool = PgPool::connect_lazy_with(config.database.with_db());

    let email_client = create_email_client(&config);

    let app_state = AppState {
        db_pool: pool,
        email_client,
    };

    let router = get_router(app_state);

    axum::serve(listener, router).await
}
