use tokio::net::TcpListener;
use z2p::{configuration::get_configuration, routes::router, startup::listener};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("Failed to read Configuration");

    let listener = listener(config.application_port).await;
    let app = router();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
