use crate::routes_path::{AdminDashboard, Login};
use crate::session_state::{TypedSession, save_session};
use crate::utils::auth_extractors::{AuthError, Credentials};
use crate::utils::authentication::validate_credentials;
use anyhow::Context;
use axum::{Form, response::Redirect};
use axum::{extract::State, response::IntoResponse};
use axum_messages::Messages;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::Repository;
use secrecy::SecretString;

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum LoginError {
    #[error("Authentication Failed.")]
    AuthError(#[from] AuthError),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(serde::Deserialize)]
pub struct LoginFormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, repo, flash, session),
    fields(username = tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    _: Login,
    session: TypedSession,
    flash: Messages,
    State(repo): State<Repository>,
    Form(form): Form<LoginFormData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // this is to simplify and consolidate the happy path
    // I am propagating errors here so my error path at bottom of this variable handler can deal with that.
    let try_response: Result<_, LoginError> = async move {
        let credentials = Credentials {
            username: form.username,
            password: form.password,
        };

        let username = credentials.username.clone();
        tracing::Span::current().record("username", tracing::field::display(&username));
        let user_id = validate_credentials(&repo, credentials).await?;
        tracing::Span::current().record("user_id", tracing::field::display(&user_id));

        session
            .cycle_id()
            .await
            .context("Failed to cycle session ID.")?;

        save_session(&session, &user_id, &username)
            .await
            .context("Failed to save session")?;

        Ok(user_id)
    }
    .await;

    if let Err(e) = try_response {
        flash.error(e.to_string());
        Err(Redirect::to(&Login.to_string()))
    } else {
        Ok(Redirect::to(&AdminDashboard.to_string()))
    }
}
