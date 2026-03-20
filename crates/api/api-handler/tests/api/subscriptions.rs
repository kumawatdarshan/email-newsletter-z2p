use crate::helpers::{FakeData, ResponseAssertions, TestAppRequests, spawn_app_testing};
use anyhow::Ok;
use api_handler::routes_path::SUBSCRIPTIONS;
use axum::http::StatusCode;

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() -> anyhow::Result<()> {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let body = app.fake_email();

    app.mock_mail_server(StatusCode::OK).await;

    app.post(SUBSCRIPTIONS)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?
        .assert_status(StatusCode::CREATED);

    Ok(())
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() -> anyhow::Result<()> {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let body = app.fake_email();

    app.mock_mail_server(StatusCode::OK).await;

    app.post(SUBSCRIPTIONS)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?
        .assert_status(StatusCode::CREATED);

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
    Ok(())
}

#[tokio::test]
async fn subscribe_returns_422_when_invalid_fields() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let test_cases = [
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (body, error) in test_cases {
        app.post(SUBSCRIPTIONS)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await?
            .assert_status_with_msg(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!(
                    "Api should fail with {} when missing data.\n{error}",
                    StatusCode::UNPROCESSABLE_ENTITY
                ),
            );
    }

    Ok(())
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_link_for_valid_data() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    app.mock_mail_server(StatusCode::OK).await;

    app.post(SUBSCRIPTIONS)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(app.fake_email())
        .send()
        .await?;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.retrieve_links(email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plaintext);
    Ok(())
}

#[tokio::test]
async fn subscribe_fails_if_there_is_fatal_db_error() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    // Sabotaging the db
    sqlx::query!("DROP TABLE subscription_tokens")
        .execute(&app.db_pool)
        .await
        .unwrap();

    app.post(SUBSCRIPTIONS)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(app.fake_email())
        .send()
        .await?
        .assert_status(StatusCode::INTERNAL_SERVER_ERROR);

    Ok(())
}
