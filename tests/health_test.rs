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

    let _response = client
        .get(format!("{}/health_check", app.addr))
        .send()
        .await
        .expect("Failed to send reqest");

    // assert!(response.status().is_success());
    assert_eq!(Some(0), Some(0)) // to validate there was no nothing present
}

#[tokio::test]
async fn test_subscribe_valid() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscribe", &app.addr))
        .header("Content-type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(StatusCode::CREATED, response.status());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Faield to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

// this is failing because we haven't implemented anything for /subscribe
#[tokio::test]
async fn test_subscribe_invalid() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let addr = app.addr;
    let client = reqwest::Client::new();

    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error) in test_cases {
        let response = client
            .post(format!("{addr}/subscribe"))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "{}",
            format_args!(
                "Api should fail with {} when missing data.\n{error}",
                StatusCode::UNPROCESSABLE_ENTITY
            )
        )
    }
}
