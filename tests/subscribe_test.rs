use axum::http::StatusCode;
use z2p::startup::spawn_app_testing;

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
