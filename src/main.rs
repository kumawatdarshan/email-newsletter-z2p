use sqlx::PgPool;
use tokio::net::TcpListener;
use z2p::{
    configuration::{AppState, get_configuration},
    routes::get_router,
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = get_configuration().expect("Failed to read Configuration");
    let connection_url = settings.database.connection_string();

    let pool = PgPool::connect(&connection_url)
        .await
        .expect("Failed to connect to Postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", settings.application_port)).await?;

    let app_state = AppState {
        db_pool: pool.clone(),
    };
    let router = get_router(app_state.into());

    axum::serve(listener, router).await
}
