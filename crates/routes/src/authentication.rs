use anyhow::{Context, anyhow};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::header::HeaderMap;
use base64::Engine;
use secrecy::{ExposeSecret, SecretString};
use sqlx::SqlitePool;
use telemetry::spawn_blocking_with_tracing;

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
    skip(credentials, pool))]
pub async fn validate_credentials(
    credentials: Credentials,
    pool: &SqlitePool,
) -> Result<String, AuthError> {
    let stored_credentials = get_stored_credentials(&credentials.username, pool).await;

    let (user_id, expected_pw_hash) = match stored_credentials {
        Ok(Some((id, hash))) => (Some(id), hash),
        // if the query succeeded but found nothing.
        Ok(None) => (
            None,
            SecretString::new("$argon2id$v=19$m=15000,t=2,p=1$...".into()), // we are doing this to force compute to avoid timing attack
        ),
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

#[tracing::instrument(name = "Get Stored Credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &SqlitePool,
) -> anyhow::Result<Option<(String, SecretString)>> {
    let row = sqlx::query!(
        r#"
           SELECT user_id, password_hash
           from users
           WHERE username = $1
        "#,
        username,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to perform a query to retrieve stored credentials.")?
    .map(|row| (row.user_id, SecretString::new(row.password_hash.into())));

    Ok(row)
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

pub(crate) fn basic_authentication(headers: HeaderMap) -> Result<Credentials, AuthError> {
    // This indirect fn is used because `AuthError::UnexpectedError` implements `From` trait for `anyhow::Error`
    // But this is very obviously `AuthError::InvalidCredentials` so in turn I am calling .map_err at this fn's call site
    // which very conviniently is this super fn `basic_authentication`.
    fn parse_credentials(headers: HeaderMap) -> anyhow::Result<Credentials> {
        let header_value = headers
            .get("Authorization")
            .context("The 'Authorization' header was missing")?
            .to_str()
            .context("The 'Authorization' header contains invalid UTF8.")?;

        let base64encoded_segment = header_value
            .strip_prefix("Basic ")
            .context("The authorization scheme is not 'Basic'")?;

        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(base64encoded_segment)
            .context("Failed to base64-decode 'Basic' credentials.")?;

        // can this utf8 error still occur
        // ofcourse it can, first we only checked for the raw header value, which is base64 encoded.
        // this time we are checking for the decoded value
        let decoded_cred =
            String::from_utf8(decoded_bytes).context("The decoded credential invalid UTF8.")?;

        let mut creds = decoded_cred.splitn(2, ':');

        let username = creds
            .next()
            .ok_or_else(|| anyhow!("A username must be provided in 'Basic' auth."))?;

        let password = creds
            .next()
            .ok_or_else(|| anyhow!("A password must be provided in 'Basic' auth."))?;

        Ok(Credentials {
            username: username.into(),
            password: password.into(),
        })
    }

    parse_credentials(headers).map_err(AuthError::InvalidCredentials)
}
