use api_handler::routes_path::{ADMIN_PASSWORD, LOGIN};
use reqwest::StatusCode;
use sqlx::types::Uuid;

use crate::helpers::{ResponseAssertions, TestAppRequests, spawn_app_testing};

#[tokio::test]
async fn must_be_logged_in_to_see_change_password_form() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;

    app.get(ADMIN_PASSWORD)
        .send()
        .await?
        .assert_redirect_to(LOGIN);

    Ok(())
}

#[tokio::test]
async fn must_be_logged_in_to_change_password() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    let uuid = Uuid::new_v4().to_string();

    let response = app
        .post(ADMIN_PASSWORD)
        .form(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &uuid,
            "new_password_check": &uuid,
        }))
        .send()
        .await?;

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());

    Ok(())
}

#[tokio::test]
async fn passwords_must_match() -> anyhow::Result<()> {
    let app = spawn_app_testing().await?;
    let pw = Uuid::new_v4().to_string();
    let pw2 = Uuid::new_v4().to_string();

    // Act - Part 1 - Login
    app.post(LOGIN)
        .form(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password
        }))
        .send()
        .await?;

    // Act - Part 2 - Try to change password
    app.post(ADMIN_PASSWORD)
        .form(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &pw,
            "new_password_check": &pw2,
        }))
        .send()
        .await?
        .assert_redirect_to(ADMIN_PASSWORD);

    let html = app.get(ADMIN_PASSWORD).send().await?.text().await?;

    assert!(html.contains(
        "<p><i>You entered two different new passwords - the field values must match.</i></p>"
    ));

    Ok(())
}
