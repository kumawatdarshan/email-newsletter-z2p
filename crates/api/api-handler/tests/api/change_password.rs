use api_handler::routes_path::{ADMIN_PASSWORD, LOGIN};
use reqwest::StatusCode;
use sqlx::types::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app_testing};

#[tokio::test]
async fn must_be_logged_in_to_see_change_password_form() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let response = app.get_change_password().await;

    assert_is_redirect_to(&response, LOGIN);
}

#[tokio::test]
async fn must_be_logged_in_to_change_password() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let uuid = Uuid::new_v4().to_string();

    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &uuid,
            "new_password_check": &uuid,
        }))
        .await;

    assert_eq!(StatusCode::UNAUTHORIZED, response.status());
}

#[tokio::test]
async fn passwords_must_match() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");
    let pw = Uuid::new_v4().to_string();
    let pw2 = Uuid::new_v4().to_string();

    // Act - Part 1 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    // Act - Part 2 - Try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &pw,
            "new_password_check": &pw2,
        }))
        .await;

    assert_is_redirect_to(&response, ADMIN_PASSWORD);

    let html = app.get_change_password_html().await;
    assert!(html.contains(
        "<p><i>You entered two different new passwords - the field values must match.</i></p>"
    ));
}
