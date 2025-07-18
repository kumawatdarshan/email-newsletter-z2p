use tokio::net::TcpListener;

pub mod routes;

/// Port to run our applicaion on.
pub async fn listener(port: u16) -> TcpListener {
    match TcpListener::bind(format!("127.0.0.1:{port}")).await {
        Ok(x) => x,
        Err(err) => {
            panic!("{err}\nUnable to bind to the port {port}")
        }
    }
}
