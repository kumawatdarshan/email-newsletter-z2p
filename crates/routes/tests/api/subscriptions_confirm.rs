use crate::helpers::spawn_app_testing;
use axum::http::StatusCode;

#[tokio::test]
async fn confirmations_without_tokens_are_rejected_with_a_400() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let response = reqwest::get(format!("{}/subscribe/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(StatusCode::BAD_REQUEST, response.status());
}

#[tokio::test]
async fn link_returned_by_subscribe_returns_a_200() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let body = app.fake_body();

    app.mock_mail_server(StatusCode::OK).await;
    app.post_subscriptions(body).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.retrieve_links(email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn clicking_on_confirmation_link_confirms_subscription() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let body = app.fake_body();

    app.mock_mail_server(StatusCode::OK).await;
    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.retrieve_links(email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "confirmed");
}
