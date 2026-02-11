use anyhow::{Context, anyhow};
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Basic;
use repository::Repository;
use repository::authentication::AuthenticationRepository;
use secrecy::{ExposeSecret, SecretString};
use std::sync::OnceLock;
use telemetry::spawn_blocking_with_tracing;

static DUMMY_HASH: OnceLock<SecretString> = OnceLock::new();

// we shouldn't even be implementing IntoResponse for these errors.
// as they are not part of http req fns
#[derive(thiserror::Error, Debug)]
pub(crate) enum AuthError {
    #[error("Invalid Credentials")]
    InvalidCredentials(#[source] anyhow::Error),

    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

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
pub async fn validate_credentials(
    repo: &Repository,
    credentials: Credentials,
) -> Result<String, AuthError> {
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

pub(crate) fn basic_authentication(
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
