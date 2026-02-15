use crate::routes_path::{ADMIN_DASHBOARD, LOGIN};
use crate::session_state::{TypedSession, save_session};
use anyhow::Context;
use axum::{Form, response::Redirect};
use axum::{extract::State, response::IntoResponse};
use axum_messages::Messages;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::Repository;
use repository::signup::SignUpRepository;
use secrecy::{ExposeSecret, SecretString};

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
enum SignUpError {
    #[error("Username Exists.")]
    UsernameExists(String),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(serde::Deserialize)]
pub struct SignUpFormData {
    username: String,
    // Tied to a User,
    user_password: SecretString,
    // Hardcoded value, required to create a user.
    master_password: SecretString,
}

#[tracing::instrument(
    skip(form, repo, flash, session),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub async fn signup(
    session: TypedSession,
    flash: Messages,
    State(repo): State<Repository>,
    Form(form): Form<SignUpFormData>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let try_response: Result<_, SignUpError> = async move {
        // check to see only master_password user can create accounts
        if form.master_password.expose_secret() != "hardcoded_master_password" {
            return Err(anyhow::anyhow!("Invalid master password").into());
        }

        tracing::Span::current().record("username", tracing::field::display(&form.username));

        let user_id = repo
            .add_new_user(&form.username, form.user_password)
            .await
            .map_err(|e| match e {
                repository::signup::SignUpError::UsernameExists => {
                    SignUpError::UsernameExists(form.username.clone())
                }
                repository::signup::SignUpError::DatabaseError(db_err) => {
                    anyhow::Error::from(db_err).into()
                }
            })?;

        tracing::Span::current().record("user_id", tracing::field::display(&user_id));

        session
            .cycle_id()
            .await
            .context("Failed to cycle session ID.")?;

        save_session(&session, &user_id, &form.username)
            .await
            .context("Failed to save session")?;

        Ok(user_id)
    }
    .await;

    match try_response {
        Err(e) => {
            flash.error(e.to_string());
            Err(Redirect::to(LOGIN))
        }
        Ok(_) => {
            flash.success("Account created successfully!");
            Ok(Redirect::to(ADMIN_DASHBOARD))
        }
    }
}
