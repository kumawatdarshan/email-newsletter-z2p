use sqlx::PgPool;
use tokio::net::TcpListener;
use z2p::{
    configuration::{AppState, get_configuration},
    routes::get_router,
    telemetry::{get_subscriber, init_subscriber},
};

/// it isn't. [here is the flake and repo](https://github.com/darshanCommits/email-newsletter-z2p/blob/master/flake.nix)
#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "debug".into(), std::io::stdout)?;
    init_subscriber(subscriber)?;

    let settings = get_configuration().expect("Failed to read Configuration");

    let pool = PgPool::connect_lazy_with(settings.database.with_db());

    let listener = TcpListener::bind(format!(
        "{}:{}",
        settings.application.host, settings.application.port
    ))
    .await?;

    let app_state = AppState {
        db_pool: pool.clone(),
    };
    let router = get_router(app_state);

    axum::serve(listener, router).await
}
