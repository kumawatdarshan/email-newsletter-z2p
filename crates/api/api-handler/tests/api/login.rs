use api_handler::routes_path;
use reqwest::StatusCode;

use crate::helpers::FakeData;
use crate::helpers::spawn_app_testing;

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

fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
