use axum::{http::StatusCode, routing::get, Router};

#[tokio::main]
async fn main() {
    let port = 3000;
    let app = Router::new()
        .route("/health_check", get(health_check));

    let listener = match tokio::net::TcpListener::bind(format!("127.0.0.1:{port}")).await {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{err}\nSomething went wrong. Unable to bind to port {port}");
            return;
        }
    };

    match axum::serve(listener, app).await {
        Ok(()) => {
            println!("Listening on port {port}");
        }
        Err(err) => {
            eprintln!("{err}\nUnable to serve at {port}");
            return;
        }
    };
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
