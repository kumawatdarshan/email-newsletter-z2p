use crate::helpers::{FakeData, spawn_app_testing};
use anyhow::Context;
use axum::http::StatusCode;

#[tokio::test]
async fn confirmations_without_tokens_are_rejected_with_a_400() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    let response = reqwest::get(format!("{}/subscriptions/confirm", app.address)).await?;

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
    Ok(())
}

#[tokio::test]
async fn link_returned_by_subscribe_returns_a_200() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    let body = app.fake_email();

    app.mock_mail_server(StatusCode::OK).await;
    app.post_subscriptions(body).await;

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .context("Got no requests")?[0];
    let confirmation_links = app.retrieve_links(email_request)?;

    let response = reqwest::get(confirmation_links.html).await?;

    assert_eq!(StatusCode::OK, response.status());

    Ok(())
}

#[tokio::test]
async fn clicking_on_confirmation_link_confirms_subscription() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    let body = app.fake_email();

    app.mock_mail_server(StatusCode::OK).await;
    app.post_subscriptions(body).await;
    let email_request = &app
        .email_server
        .received_requests()
        .await
        .context("Got no requests")?[0];

    let confirmation_links = app.retrieve_links(email_request)?;

    reqwest::get(confirmation_links.html)
        .await?
        .error_for_status()?;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .context("Failed to fetch saved subscriptions")?;

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
    Ok(())
}
