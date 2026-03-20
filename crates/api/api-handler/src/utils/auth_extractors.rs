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

#[derive(Debug, Clone)]
pub(crate) struct Credentials {
    pub(crate) username: String,
    pub(crate) password: SecretString,
}

/// JSON error, typical usage being POST/DELETE/PUT requests
/// Common exception being server event to client such as `Session Expired, re-login`
/// You must have repo as extractor to use this
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub username: String,
    pub user_id: String,
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
    Repository: axum::extract::FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let session = TypedSession::from_request_parts(parts, state)
            .await
            .map_err(|e| AuthError::UnexpectedError(anyhow!("Session error: {:?}", e)))?;

        if let Ok(user) = try_session(&session).await {
            return Ok(user);
        }

        Err(AuthError::InvalidCredentials(anyhow!(
            "No valid credentials provided"
        )))
    }
}

/// Redirects on 401, typical usage being protected GET endpoints
pub struct RequireAuth(pub AuthenticatedUser);

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
    Repository: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        match AuthenticatedUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(RequireAuth(user)),
            Err(_) => Err(Redirect::to(LOGIN).into_response()),
        }
    }
}

/// Try to authenticate via session cookie.
async fn try_session(session: &TypedSession) -> Result<AuthenticatedUser, AuthError> {
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

    Ok(AuthenticatedUser { user_id, username })
}
