use crate::helpers::{ConfirmationLinks, FakeData, TestApp, spawn_app_testing};
use axum::http::StatusCode;
use sqlx::types::Uuid;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create create_unconfirmed_subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(app.fake_email())
        .await
        .error_for_status()
        .unwrap();

    let email_request = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.retrieve_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        // assertion that no request is made
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = app.fake_newsletter();

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn newsletter_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = app.fake_newsletter();

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(StatusCode::OK, response.status());
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let test_cases = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "Newsletter body as plain text",
                    "html": "<p>Newsletter body as HTML</p>",
                }
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {error_message}."
        );
    }
}

#[tokio::test]
async fn requests_missing_auth_are_rejected() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", app.address))
        .json(&app.fake_newsletter())
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let user = Uuid::new_v4().to_string();
    let pw = Uuid::new_v4().to_string();

    let newsletter_request_body = app.fake_newsletter();

    let response = app
        .post_newsletters_with_auth(newsletter_request_body, &user, &pw)
        .await;

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn invalid_password_is_rejected() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let user = &app.test_user.username;

    let pw = Uuid::new_v4().to_string();
    assert_ne!(pw, app.test_user.password);

    let newsletter_request_body = app.fake_newsletter();

    let response = app
        .post_newsletters_with_auth(newsletter_request_body, user, &pw)
        .await;

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}
