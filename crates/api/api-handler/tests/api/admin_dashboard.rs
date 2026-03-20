use anyhow::Ok;
use api_handler::routes_path::{ADMIN_DASHBOARD, LOGIN};

use crate::helpers::{ResponseAssertions, TestAppRequests, spawn_app_testing};

#[tokio::test]
async fn must_be_logged_in_to_access_admin_dashboard() -> anyhow::Result<()> {
    let app = spawn_app_testing().await.expect("Failed to spawn app");

    app.get(ADMIN_DASHBOARD)
        .send()
        .await?
        .assert_redirect_to(LOGIN);

    Ok(())
}
