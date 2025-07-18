use axum::http::StatusCode;

// TODO: this route should check all of our services' health
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
