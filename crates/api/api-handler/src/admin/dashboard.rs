use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

use crate::routes::routes_path::AdminDashboard;
use crate::utils::auth_extractors::RequireAuth;

pub async fn admin_dashboard(
    _: AdminDashboard,
    RequireAuth(user): RequireAuth,
) -> Result<Response, Response> {
    let username = user.username;

    let html = format!(
        r#"
            <!DOCTYPE html>
<html lang="en">

<head>
  <meta charset="UTF-8">
  <title>Admin Dashboard</title>
</head>

<body>
  <h1>Welcome! {username}</h1>
</body>

</html>
        "#
    );

    Ok((StatusCode::OK, Html(html)).into_response())
}
