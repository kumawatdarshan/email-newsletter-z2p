use axum::{Router, http::StatusCode, routing::get};
use tokio::net::TcpListener;

/// Port to run our applicaion on.
pub async fn listener(port: u16) -> TcpListener {
    match TcpListener::bind(format!("127.0.0.1:{port}")).await {
        Ok(x) => x,
        Err(err) => {
            panic!("{err}\nUnable to bind to the port {port}")
        }
    }
}

pub fn routes() -> Router {
    Router::new().route("/health_check", get(health_check))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
