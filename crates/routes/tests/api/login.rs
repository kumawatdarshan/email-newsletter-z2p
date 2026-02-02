use crate::helpers::FakeData;
use crate::helpers::assert_is_redirect_to;
use crate::helpers::spawn_app_testing;

#[tokio::test]
async fn an_error_flash_message_is_sent_on_failure() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let login_body = app.fake_invalid_account();

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/login");

    let flash_cookie = response.cookies().find(|x| x.name() == "_flash").unwrap();
    let flash_cookie = urlencoding::decode(flash_cookie.value())?;
    assert_eq!(flash_cookie, "Authentication Failed.");

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
