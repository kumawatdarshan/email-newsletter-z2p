use z2p::{run, PORT};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{PORT}")).await?;
    run(listener).await
}
