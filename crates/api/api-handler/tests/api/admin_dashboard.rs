use api_handler::routes_path::Login;

use crate::helpers::{assert_is_redirect_to, spawn_app_testing};

#[tokio::test]
async fn must_be_logged_in_to_access_admin_dashboard() {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    let response = app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, &Login.to_string());
}
