use axum::{extract::Query, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
#[axum::debug_handler]
pub async fn confirm(
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(StatusCode::OK)
}
