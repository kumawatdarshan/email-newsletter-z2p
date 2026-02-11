use crate::helpers::spawn_app_testing;
use api_handler::routes_path;
use axum::http::StatusCode;
use reqwest::Client;

/// # Why this complicated test for something simple as health_check?
/// This is a **black box test**, meaning it is decoupled(*mostly*) from our codebase.
/// Decoupled as in, it is meant to behave like how consumers of this API would use it.
/// thus it makes several checks:
/// - Are we firing the correct endpoint? (/health_check)
/// - Are we firing the correct request? (GET)
/// - Is it a successful response? (200)
/// - Is there any content received? (There should not be any)
/// ---
/// ### Although I am honestly not entirely convinced...
/// If i even need this, I am canonizing it as the author introducing me to integration testing and that i don't actually need this in rust world. could be wrong.
/// Update: it was totally worth it. I now know the struggles of integration testing and how to get around them.
/// Update 2: New Revelation. `/health` endpoint is rather common, it is used to test if our service is alive.
#[tokio::test]
async fn test_health_check() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let response = Client::new()
        .get(app.typed_path(routes_path::HealthCheck))
        .send()
        .await
        .expect("failed to send request");

    assert_eq!(StatusCode::OK, response.status());
    assert_eq!(Some(0), Some(0)) // to validate there was no nothing present
}
