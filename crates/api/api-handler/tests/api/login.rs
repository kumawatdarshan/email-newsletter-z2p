use crate::helpers::{FakeData, ResponseAssertions, TestAppRequests, spawn_app_testing};
use api_handler::routes_path::{ADMIN_DASHBOARD, LOGIN};

#[tokio::test]
async fn an_error_flash_message_is_sent_on_failure() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    app.post(LOGIN)
        .form(&app.fake_invalid_account())
        .send()
        .await?
        .assert_redirect_to(LOGIN);

    // check html if the flash msg is appearing
    let html = app.get(LOGIN).send().await?.text().await?;
    assert!(
        html.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Auth failed such html should appear."
    );

    // reload
    let html = app.get(LOGIN).send().await?.text().await?;
    assert!(
        !html.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Page is reloaded, no auth failed msg should appear."
    );

    Ok(())
}

#[tokio::test]
async fn redirect_to_admin_dashboard_on_login_success() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });

    app.post(LOGIN)
        .form(&login_body)
        .send()
        .await?
        .assert_redirect_to(ADMIN_DASHBOARD);

    let html = app.get(ADMIN_DASHBOARD).send().await?.text().await?;

    assert!(
        html.contains(&format!("Welcome! {}", &app.test_user.username)),
        "Dashboard should greet the logged-in user."
    );

    Ok(())
}
