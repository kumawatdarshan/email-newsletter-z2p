use anyhow::{Context, anyhow};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Basic},
};
use newsletter_macros::IntoErrorResponse;
use repository::{Repository, authentication::AuthenticationRepository};
use secrecy::{ExposeSecret, SecretString};
use std::sync::OnceLock;
use telemetry::spawn_blocking_with_tracing;

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub username: String,
}

#[axum::debug_middleware]
pub async fn require_authentication(
    State(repo): State<Repository>,
    auth_header: Option<TypedHeader<Authorization<Basic>>>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let credentials = basic_authentication(auth_header)?;

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(&repo, credentials.clone()).await?;

    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    // Store authenticated user in request extensions
    request.extensions_mut().insert(AuthenticatedUser {
        user_id,
        username: credentials.username,
    });

    Ok(next.run(request).await)
}

#[derive(thiserror::Error, Debug, IntoErrorResponse)]
pub(crate) enum AuthError {
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

/// Returns user_id
/// # Errors:
/// If no such user found: AuthError::InvalidCredentials
/// If other error: AuthError::UnexpectedError
#[tracing::instrument(
    name = "Validate Credentials"
    skip(credentials, repo))]
pub(crate) async fn validate_credentials(
    repo: &Repository,
    credentials: Credentials,
) -> Result<String, AuthError> {
    static DUMMY_HASH: OnceLock<SecretString> = OnceLock::new(); // should this exist outside?? idk
    let stored_credentials = repo
        .get_stored_credentials(&credentials.username)
        .await
        .context("Failed to perform a query to retrieve stored credentials.");

    let (user_id, expected_pw_hash) = match stored_credentials {
        Ok(Some((id, hash))) => (Some(id), hash),
        // if the query succeeded but found nothing. For timing attack
        Ok(None) => {
            let dummy = DUMMY_HASH.get_or_init(|| {
                let salt =
                    SaltString::encode_b64(b"pixel_2_xl_is_the_best_phone").expect("Invalid Salt");
                let phc = Argon2::default()
                    .hash_password(b"pokemon_is_the_best_nintendo_ip", &salt)
                    .expect("Failed to compute dummy hash")
                    .to_string();
                SecretString::new(phc.into())
            });
            (None, dummy.clone())
        }
        // if the query failed.
        Err(e) => return Err(AuthError::UnexpectedError(e)),
    };

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_pw_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(AuthError::UnexpectedError)??; // Propagate AuthError from verify_password_hash

    // If user_id not found in the sql query, error out.
    user_id.ok_or_else(|| AuthError::InvalidCredentials(anyhow!("Unknown Username")))
}

#[tracing::instrument(
    name = "Verify Password Hash"
    skip(expected_pw_hash, pw_candidate))]
fn verify_password_hash(
    expected_pw_hash: SecretString,
    pw_candidate: SecretString,
) -> Result<(), AuthError> {
    let expected_pw_hash = PasswordHash::new(expected_pw_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(AuthError::UnexpectedError)?;

    Argon2::default()
        .verify_password(pw_candidate.expose_secret().as_bytes(), &expected_pw_hash)
        .context("Invalid Password")
        .map_err(AuthError::InvalidCredentials)
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
