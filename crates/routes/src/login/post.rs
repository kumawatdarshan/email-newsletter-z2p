use std::sync::Arc;

use axum::{
    Form,
    body::Body,
    extract::State,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use newsletter_macros::{DebugChain, IntoErrorResponse};
use secrecy::SecretString;
use state::AppState;

use crate::authentication::{AuthError, Credentials, validate_credentials};

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum LoginError {
    #[error("Authentication failed.")]
    #[status(StatusCode::UNAUTHORIZED)]
    #[headers([header::LOCATION = "/login"])]
    AuthError(#[source] crate::authentication::AuthError),

    #[error("Something went wrong.")]
    #[status(StatusCode::SEE_OTHER)]
    #[headers([header::LOCATION = "/login"])]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, state),
    fields(username = tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &state.db_pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .body(Body::empty())
        .unwrap())
}
