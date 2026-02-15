use super::authentication::validate_credentials;
use crate::routes_path::Login;
use crate::session_state::TypedSession;
use anyhow::anyhow;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Basic},
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

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
}

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
            Err(_) => Err(Redirect::to(&Login.to_string()).into_response()),
        }
    }
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

        // 1. Try Session - If it fails or is empty, we just move on
        if let Ok(user) = try_session(&session).await {
            return Ok(user);
        }

        // 2. Try Authenticating
        if let Ok(user) = try_basic_auth(parts, state, session).await {
            return Ok(user);
        }

        // 3. Everything failed
        Err(AuthError::InvalidCredentials(anyhow!(
            "No valid credentials provided"
        )))
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

/// Try to authenticate via Basic Auth header.
async fn try_basic_auth<S>(
    parts: &mut axum::http::request::Parts,
    state: &S,
    session: TypedSession,
) -> Result<AuthenticatedUser, AuthError>
where
    S: Send + Sync,
    Repository: axum::extract::FromRef<S>,
{
    // check if auth header is available
    let auth_header = Option::<TypedHeader<Authorization<Basic>>>::from_request_parts(parts, state)
        .await
        .map_err(|e| AuthError::UnexpectedError(anyhow::anyhow!("Header error: {:?}", e)))?;

    let credentials = basic_authentication(auth_header)?;
    let username = credentials.username.clone();
    let repo = Repository::from_ref(state);
    let user_id = validate_credentials(&repo, credentials).await?;

    save_session(session, &user_id, &username).await?;

    Ok(AuthenticatedUser { user_id, username })
}

fn basic_authentication(
    auth_header: Option<TypedHeader<Authorization<Basic>>>,
) -> Result<Credentials, AuthError> {
    let TypedHeader(auth) = auth_header
        .ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Missing credentials")))?;

    let username = auth.username();
    let password = auth.password();

    if username.is_empty() || password.is_empty() {
        return Err(AuthError::InvalidCredentials(anyhow::anyhow!(
            "Username or password cannot be empty"
        )));
    }

    Ok(Credentials {
        username: username.into(),
        password: password.into(),
    })
}

pub async fn save_session(
    session: TypedSession,
    user_id: &str,
    username: &str,
) -> Result<(), AuthError> {
    session
        .insert_user_id(user_id)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;
    session
        .insert_username(username)
        .await
        .map_err(|e| AuthError::UnexpectedError(e.into()))?;

    Ok(())
}
