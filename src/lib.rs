use axum::{
    Form, Router,
    http::StatusCode,
    routing::{get, post},
};
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
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(serde::Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn subscribe(Form(form_data): Form<FormData>) -> StatusCode {
    StatusCode::OK
}
