use crate::helpers::spawn_app_testing;
use axum::http::StatusCode;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(StatusCode::CREATED, response.status());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscriptions(body.into()).await;

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
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(body["html"].as_str().unwrap());

    let text_link = get_link(body["text"].as_str().unwrap());

    assert_eq!(html_link, text_link);
}
