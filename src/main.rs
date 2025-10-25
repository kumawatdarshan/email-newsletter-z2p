use sqlx::PgPool;
use z2p::{app_state::AppFactory, configuration::AppState, routes::get_router};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_factory = AppFactory::new(false)?.init_subscriber()?;
    let listener = app_factory.create_listener().await?;

    let pool = PgPool::connect_lazy_with(app_factory.config.database.with_db());
    let email_client = app_factory.create_email_client();

    let app_state = AppState {
        db_pool: pool,
        email_client,
    };

    let router = get_router(app_state);

    axum::serve(listener, router).await
}
