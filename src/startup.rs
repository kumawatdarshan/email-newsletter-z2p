use tokio::net::TcpListener;

pub async fn listener(port: u16) -> TcpListener {
    let address = format!("127.0.0.1:{port}");

    match TcpListener::bind(address).await {
        Ok(x) => x,
        Err(err) => {
            eprintln!("{err}\nUnable to bind to the port {port}");
            panic!();
        }
    }
}
