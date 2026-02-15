use crate::helpers::FakeData;
use crate::helpers::assert_is_redirect_to;
use crate::helpers::spawn_app_testing;
use api_handler::routes_path;

#[tokio::test]
async fn an_error_flash_message_is_sent_on_failure() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let login_body = app.fake_invalid_account();

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, &routes_path::Login.to_string());

    let html_page = app.get_login_html().await;
    assert!(
        html_page.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Auth failed such html should appear."
    );

    // reload
    let html_page = app.get_login_html().await;
    assert!(
        !html_page.contains(r#"<p><i>Authentication Failed.</i></p>"#),
        "Page is reloaded, no auth failed msg should appear."
    );

    Ok(())
}

#[tokio::test]
async fn redirect_to_admin_dashboard_on_login_success() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let login_body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    });

    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, &routes_path::AdminDashboard.to_string());

    let html_page = app.get_admin_dashboard_html().await;
    println!("{html_page}");

    assert!(html_page.contains(&format!("Welcome! {}", &app.test_user.username)));

    Ok(())
}
