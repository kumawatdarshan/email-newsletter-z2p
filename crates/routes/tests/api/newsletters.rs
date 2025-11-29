use crate::helpers::{ConfirmationLinks, TestApp, spawn_app_testing};
use axum::http::StatusCode;
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

    app.post_subscriptions(app.fake_body())
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

    let newsletter_request_body = serde_json::json!({
       "title": "Newsletter Title",
       "content": {
           "text": "Plain-text Body",
           "html": "<p>HTML body</p>",
       }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(response.status(), StatusCode::OK);
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

    let newsletter_request_body = serde_json::json!({
       "title": "Newsletter Title",
       "content": {
           "text": "Plain-text Body",
           "html": "<p>HTML body</p>",
       }
    });

    let response = app.post_newsletters(newsletter_request_body).await;

    assert_eq!(response.status(), StatusCode::OK);
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
            400,
            response.status(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
