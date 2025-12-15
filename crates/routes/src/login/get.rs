use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};

pub async fn login_form() -> impl IntoResponse {
    // include_str will bundle the file content into the binary, avoid if the html gets large.
    (StatusCode::OK, Html(include_str!("./login.html")))
}
