use crate::helpers::{FakeData, spawn_app_testing};
use axum::http::StatusCode;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let body = app.fake_body();

    app.mock_mail_server(StatusCode::OK).await;

    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(StatusCode::CREATED, response.status());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let body = app.fake_body();

    app.mock_mail_server(StatusCode::OK).await;
    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(StatusCode::CREATED, response.status());

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Faield to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_422_when_invalid_fields() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

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

#[tokio::test]
async fn subscribe_sends_a_confirmation_link_for_valid_data() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let body = app.fake_body();

    app.mock_mail_server(StatusCode::OK).await;

    app.post_subscriptions(body).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.retrieve_links(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plaintext);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_fatal_db_error() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let body = app.fake_body();

    // Sabotaging the db
    sqlx::query!("DROP TABLE subscription_tokens")
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_subscriptions(body).await;

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
