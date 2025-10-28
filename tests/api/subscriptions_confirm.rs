use crate::helpers::spawn_app_testing;
use axum::http::StatusCode;
use wiremock::{
    Mock, ResponseTemplate,
    matchers::{method, path},
};

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
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let confirmation_links = app.get_links().await;

    let response = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(StatusCode::OK, response.status());
}
