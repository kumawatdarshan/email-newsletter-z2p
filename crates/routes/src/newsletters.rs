use std::sync::Arc;

use anyhow::{Context, anyhow};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use base64::Engine;
use domain::SubscriberEmail;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::SqlitePool;
use state::AppState;
use telemetry::spawn_blocking_with_tracing;

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum PublishError {
    #[error("Authentication failed.")]
    #[status(StatusCode::UNAUTHORIZED)]
    #[headers([header::WWW_AUTHENTICATE = r#"Basic realm="publish""#])]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, state, headers),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub(crate) async fn publish_newsletter(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let credentials = basic_authentication(headers).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = valide_credentials(credentials, &state.db_pool)
        .await?
        .unwrap();
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let subcribers = get_confirmed_subscribers(&state.db_pool).await?;

    for subscriber in subcribers {
        match subscriber {
            Ok(s) => {
                let email = &s.email;
                state
                    .email_client
                    .send_email(email, &body.title, &body.content.html, &body.content.text)
                    .await
                    .with_context(|| format!("Failed to send newsletter to: {email}"))?;
            }
            // We did that convoluted Result<Vec<Result<X>,E>, E> so we could do this
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber, Using an invalid mail.",
                )
            }
        }
    }

    Ok(StatusCode::OK)
}

struct Credentials {
    username: String,
    password: SecretString,
}

fn basic_authentication(headers: HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string.")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme is not 'Basic'")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials.")?;

    let decoded_cred = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8.")?; // can this utf8 error still occur

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

#[tracing::instrument(name = "Get Confirmed Subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &SqlitePool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|x| {
            SubscriberEmail::parse(x.email)
                .map(|email| ConfirmedSubscriber { email })
                .map_err(|error| anyhow!(error))
        })
        .collect();

    Ok(confirmed_subscribers)
}

#[tracing::instrument(name = "Get Stored Credentials", skip(username, pool))]
async fn get_stored_credentials(
    username: &str,
    pool: &SqlitePool,
) -> Result<Option<(Option<String>, SecretString)>, anyhow::Error> {
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
    name = "Validate Credentials"
    skip(credentials, pool))]
async fn valide_credentials(
    credentials: Credentials,
    pool: &SqlitePool,
) -> Result<Option<String>, PublishError> {
    let mut user_id = None;
    let mut expected_pw_hash: SecretString = "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
        .into();

    if let Some((stored_user_id, stored_pw_hash)) =
        get_stored_credentials(&credentials.username, pool)
            .await
            .map_err(PublishError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_pw_hash = stored_pw_hash;
    }

    spawn_blocking_with_tracing(move || {
        verify_password_hash(expected_pw_hash, credentials.password)
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(PublishError::UnexpectedError)??;

    // If user_id not found in the sql query, error out.
    user_id.ok_or_else(|| PublishError::AuthError(anyhow!("Unknown Username")))
}

#[tracing::instrument(
    name = "Verify Password Hash"
    skip(expected_pw_hash, pw_candidate))]
fn verify_password_hash(
    expected_pw_hash: SecretString,
    pw_candidate: SecretString,
) -> Result<(), PublishError> {
    let expected_pw_hash = PasswordHash::new(expected_pw_hash.expose_secret())
        .context("Failed to parse hash in PHC string format.")
        .map_err(PublishError::UnexpectedError)?;

    Argon2::default()
        .verify_password(pw_candidate.expose_secret().as_bytes(), &expected_pw_hash)
        .context("Invalid Password")
        .map_err(PublishError::AuthError)
}
