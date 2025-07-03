use tokio::net::TcpListener;
use z2p::{listener, routes};

/// Why this complicated test for something simple as health_check?
/// This is a black box test, meaning it is decoupled(*mostly*) from our codebase.
/// Decoupled as in, it is meant to behave like how consumers of this API would use it.
/// thus it makes several checks:
/// 1. Are we firing the correct endpoint? (/health_check)
/// 1. Are we firing the correct request? (GET)
/// 1. Is it a successful response? (200)
/// 1. Is there any content recieved? (There should not be any)
#[tokio::test]
async fn test_health_check() {
    // Arrange

    let host = "127.0.0.1";
    let listener = listener(0).await;
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://{host}:{port}");

    spawn_app(listener).await.expect("Failed to spawn app");
    dbg!(format!("{addr}/health_check"));

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{addr}/health_check"))
        .send()
        .await
        .expect("Failed to send reqest");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length()) // to validate there was no nothing present
}

async fn spawn_app(listener: TcpListener) -> std::io::Result<()> {
    let app = routes();

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("Failed to bind address")
    });

    Ok(())
}
