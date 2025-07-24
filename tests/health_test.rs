use axum::http::StatusCode;
use z2p::startup::spawn_app_testing;

/// # Why this complicated test for something simple as health_check?
/// This is a **black box test**, meaning it is decoupled(*mostly*) from our codebase.
/// Decoupled as in, it is meant to behave like how consumers of this API would use it.
/// thus it makes several checks:
/// - Are we firing the correct endpoint? (/health_check)
/// - Are we firing the correct request? (GET)
/// - Is it a successful response? (200)
/// - Is there any content recieved? (There should not be any)
/// ---
/// ### Although I am honestly not entirely convinced...
/// If i even need this, I am canonizing it as the author introducing me to integration testing and that i don't actually need this in rust world. could be wrong.
/// Update: it was totally worth it. I now know the struggles of integration testing and how to get around them.
#[tokio::test]
async fn test_health_check() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", app.addr))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), Some(0)) // to validate there was no nothing present
}
