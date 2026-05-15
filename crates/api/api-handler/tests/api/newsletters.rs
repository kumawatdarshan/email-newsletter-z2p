use crate::helpers::{
    AuthenticatedRequest, ConfirmationLinks, FakeData, ResponseAssertions, TestApp,
    TestAppRequests, TestUser, spawn_app_testing,
};
use api_handler::routes_path::{ADMIN_NEWSLETTERS, SUBSCRIPTIONS};
use axum::http::StatusCode;
use sqlx::types::Uuid;
use std::time::Duration;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

pub async fn post_newsletter(
    app: &TestApp,
    body: &serde_json::Value,
) -> anyhow::Result<reqwest::Response> {
    let response = app.post(ADMIN_NEWSLETTERS).form(body).send().await?;

    Ok(response)
}

pub async fn get_newsletter_html(app: &TestApp) -> anyhow::Result<String> {
    let text = app.get(ADMIN_NEWSLETTERS).send().await?.text().await?;

    Ok(text)
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> anyhow::Result<ConfirmationLinks> {
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create create_unconfirmed_subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post(SUBSCRIPTIONS)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(app.fake_email())
        .send()
        .await?
        .error_for_status()?;

    let email_request = app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.retrieve_links(&email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) -> anyhow::Result<()> {
    let confirmation_link = create_unconfirmed_subscriber(app).await?;
    reqwest::get(confirmation_link.html)
        .await?
        .error_for_status()?;

    Ok(())
}

#[tokio::test]
async fn newsletter_are_not_delivered_to_unconfirmed_subscribers() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    create_unconfirmed_subscriber(&app).await?;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let app = app.login(&app.test_user).await?;

    post_newsletter(app, &app.fake_newsletter())
        .await?
        .assert_redirect_to(ADMIN_NEWSLETTERS);

    Ok(())
}

#[tokio::test]
async fn newsletter_are_delivered_to_confirmed_subscribers() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    create_confirmed_subscriber(&app).await?;

    Mock::given(path("/email"))
        .respond_with(ResponseTemplate::new(StatusCode::OK))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let app = app.login(&app.test_user).await?;

    post_newsletter(app, &app.fake_newsletter())
        .await?
        .assert_redirect_to(ADMIN_NEWSLETTERS);

    Ok(())
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    let test_cases = vec![
        (
            serde_json::json!({
                "text": "Newsletter body as plain text",
                "html": "<p>Newsletter body as HTML</p>",
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing html & text",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        app.login(&app.test_user)
            .await?
            .post(ADMIN_NEWSLETTERS)
            .form(&invalid_body)
            .send()
            .await?
            .assert_status_with_msg(
                StatusCode::UNPROCESSABLE_ENTITY,
                &format!("The API did not fail with 422 Unprocessable Entity when the payload was {error_message}."),
            );
    }
    Ok(())
}

#[tokio::test]
async fn requests_missing_auth_are_rejected() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    let response = app
        .post(ADMIN_NEWSLETTERS)
        .form(&app.fake_newsletter())
        .send()
        .await?
        .assert_status(StatusCode::UNAUTHORIZED);

    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
    Ok(())
}

#[tokio::test]
async fn non_existing_user_is_rejected() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    let response = app
        .login(&TestUser::new())
        .await?
        .post(ADMIN_NEWSLETTERS)
        .form(&app.fake_newsletter())
        .send()
        .await?
        .assert_status(StatusCode::UNAUTHORIZED);

    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );

    Ok(())
}

#[tokio::test]
async fn invalid_password_is_rejected() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    let user_with_wrong_pw = {
        let mut x = app.test_user.clone();
        let password = Uuid::new_v4().to_string();
        assert_ne!(
            password, x.password,
            "Unfortunate randomness. 2 UUID which should not match, matched. Run the test again."
        );

        x.password = password;
        x
    };

    let response = app
        .login(&user_with_wrong_pw)
        .await?
        .post(ADMIN_NEWSLETTERS)
        .form(&app.fake_newsletter())
        .send()
        .await?
        .assert_status(StatusCode::UNAUTHORIZED);

    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );

    Ok(())
}

#[tokio::test]
async fn get_responds_with_issue_form() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    let app = app.login(&app.test_user).await?;

    let html = get_newsletter_html(app).await?;

    assert!(html.contains(&format!(r#"<h1>Hello {}</h1>"#, &app.test_user.username)));
    Ok(())
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    create_confirmed_subscriber(&app).await?;

    let app = app.login(&app.test_user).await?;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_body = serde_json::json!({
       "title": "Newsletter Title",
       "text": "Plain-text Body",
       "html": "<p>HTML body</p>",
       "idempotency_key": Uuid::new_v4().to_string()
    });

    // First attempt
    post_newsletter(app, &newsletter_body)
        .await?
        .assert_redirect_to(ADMIN_NEWSLETTERS);

    let html = get_newsletter_html(app).await?;
    assert!(html.contains("<p><i>The newsletter issue has been published!</i></p>"));

    // Second attempt (Idempotency check)
    post_newsletter(app, &newsletter_body)
        .await?
        .assert_redirect_to(ADMIN_NEWSLETTERS);

    let html = get_newsletter_html(app).await?;
    assert!(html.contains("<p><i>The newsletter issue has been published!</i></p>"));

    Ok(())
}

/// ON: How to implement cross-request synchronization.
/// In-memory locks (e.g. tokio::sync::Mutex) would work if
/// all incoming requests were being served by a single API instance.
/// This is not our case: our API is replicated,
/// therefore the two requests might end up being processed by two different instances.
#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    create_confirmed_subscriber(&app).await?;

    Mock::given(path("/email"))
        .and(method("POST"))
        // 2 sec was causing timeout in my case, IDK if i ever set it up or if its defaults lol.
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(900)))
        .expect(1)
        .mount(&app.email_server)
        .await;
    let app = app.login(&app.test_user).await?;

    // intentionally share idempotency key to both
    let newsletter = app.fake_newsletter();
    let (response1, response2) = tokio::try_join!(
        post_newsletter(app, &newsletter),
        post_newsletter(app, &newsletter)
    )?;

    assert_eq!(response1.status(), response2.status());
    assert_eq!(response1.text().await?, response2.text().await?);

    Ok(())
}
