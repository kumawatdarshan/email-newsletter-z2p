use axum::{Form, response::Redirect};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use sqlx::SqlitePool;

use axum::{extract::State, response::IntoResponse};
use newsletter_macros::{DebugChain, IntoErrorResponse};
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
    skip(form, db_pool, jar),
    fields(username = tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    jar: CookieJar,
    State(db_pool): State<SqlitePool>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let jar = match validate_credentials(credentials, &db_pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", tracing::field::display(&user_id));
            jar
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let cookie = Cookie::new("_flash", e.to_string());
            jar.add(cookie)
        }
    };

    (jar, Redirect::to("/login").into_response())
}
