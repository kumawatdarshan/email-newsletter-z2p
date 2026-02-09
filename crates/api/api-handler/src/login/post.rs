use axum::{Form, response::Redirect};
use axum::{extract::State, response::IntoResponse};
use axum_messages::Messages;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::Repository;
use secrecy::SecretString;

use crate::authentication::{AuthError, Credentials, validate_credentials};

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum LoginError {
    #[error("Authentication Failed.")]
    AuthError(#[source] crate::authentication::AuthError),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, repo, message),
    fields(username = tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    message: Messages,
    State(repo): State<Repository>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    match validate_credentials(&repo, credentials).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            Ok(Redirect::to("/login").into_response())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            message.error(e.to_string());

            Err(Redirect::to("/login").into_response())
        }
    }
}
