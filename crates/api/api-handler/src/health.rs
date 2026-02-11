use crate::routes::routes_path::HealthCheck;
use axum::http::StatusCode;

// TODO: this route should check all of our services' health
pub(crate) async fn health_check(_: HealthCheck) -> StatusCode {
    StatusCode::OK
}
