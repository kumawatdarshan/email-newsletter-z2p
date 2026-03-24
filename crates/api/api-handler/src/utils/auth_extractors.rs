use std::marker::PhantomData;

use crate::routes_path::LOGIN;
use crate::session_state::TypedSession;
use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use newsletter_macros::IntoErrorResponse;
use repository::Repository;
use secrecy::SecretString;

#[derive(Debug, Clone)]
pub(crate) struct Credentials {
    pub(crate) username: String,
    pub(crate) password: SecretString,
}

#[derive(Clone, Debug)]
pub struct User {
    pub username: String,
    pub user_id: String,
}

/// Extractor returns `401` with `WWW-Authenticate` on failure.
/// Suited for API endpoints (POST / PUT / DELETE).
pub struct Api;

/// Extractor redirects to the login page on failure.
/// Suited for browser-facing GET endpoints.
pub struct Browser;

/// Authenticated user extractor, parameterised by rejection behaviour.
///
/// ```
/// // API handler – returns 401 JSON on failure
/// async fn publish(auth: Authenticated<Api>) { ... }
///
/// // Browser handler – redirects to /login on failure
/// async fn dashboard(auth: Authenticated<Browser>) { ... }
/// ```
pub struct Authenticated<T: AuthRejection> {
    pub identity: User,
    _mode: PhantomData<T>,
}

#[derive(thiserror::Error, Debug, IntoErrorResponse)]
pub enum AuthError {
    #[error("Invalid Credentials")]
    #[status(StatusCode::UNAUTHORIZED)]
    #[headers([header::WWW_AUTHENTICATE , r#"Basic realm="publish""#])]
    InvalidCredentials(#[source] anyhow::Error),

    #[error("Something went wrong.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

pub trait AuthRejection: Send + Sync + 'static {
    fn reject(err: AuthError) -> Response;
}

impl AuthRejection for Api {
    fn reject(err: AuthError) -> Response {
        err.into_response()
    }
}

impl AuthRejection for Browser {
    fn reject(_err: AuthError) -> Response {
        Redirect::to(LOGIN).into_response()
    }
}

impl<T: AuthRejection> Authenticated<T> {
    fn new(identity: User) -> Self {
        Self {
            identity,
            _mode: PhantomData,
        }
    }
}

// Deref so callers can write `auth.username` instead of `auth.identity.username`.
impl<M: AuthRejection> std::ops::Deref for Authenticated<M> {
    type Target = User;
    fn deref(&self) -> &Self::Target {
        &self.identity
    }
}

impl<S, M> FromRequestParts<S> for Authenticated<M>
where
    S: Send + Sync,
    M: AuthRejection,
    Repository: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = TypedSession::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                M::reject(AuthError::UnexpectedError(anyhow!(
                    "Session error: {:?}",
                    e
                )))
            })?;

        try_session(&session)
            .await
            .map(Authenticated::new)
            .map_err(|e| M::reject(e))
    }
}

/// Try to authenticate via session cookie.
async fn try_session(session: &TypedSession) -> Result<User, AuthError> {
    let user_id = session
        .get_user_id()
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?
        .ok_or_else(|| AuthError::InvalidCredentials(anyhow!("No user_id in session")))?;

    let username = session
        .get_username()
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?
        .ok_or_else(|| AuthError::InvalidCredentials(anyhow!("No username in session")))?;

    Ok(User { user_id, username })
}
