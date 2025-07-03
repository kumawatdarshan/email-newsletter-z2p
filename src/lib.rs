use axum::{http::StatusCode, routing::get, Router};
use tokio::net::TcpListener;

/// Port to run our applicaion on.
pub const PORT: u16 = 3000;

pub async fn run(listener: TcpListener) -> Result<(), std::io::Error> {
    let app = Router::new()
        .route("/health_check", get(health_check));

    axum::serve(listener, app).await
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
