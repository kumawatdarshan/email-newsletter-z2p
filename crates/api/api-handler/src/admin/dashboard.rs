use crate::utils::auth_extractors::RequireAuth;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};

pub async fn admin_dashboard(RequireAuth(user): RequireAuth) -> Result<Response, Response> {
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

    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
    </ol>
</body>

</html>
        "#
    );

    Ok((StatusCode::OK, Html(html)).into_response())
}
