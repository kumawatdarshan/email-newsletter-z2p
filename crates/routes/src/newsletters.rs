use axum::http::StatusCode;
use axum::response::IntoResponse;

pub(crate) async fn publish_newsletter() -> impl IntoResponse {
    StatusCode::OK
}
