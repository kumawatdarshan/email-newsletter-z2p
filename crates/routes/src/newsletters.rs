use anyhow::{Context, anyhow};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use base64::Engine;
use domain::SubscriberEmail;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::SqlitePool;
use state::AppState;
use std::sync::Arc;

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

pub(crate) async fn publish_newsletter(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let _cred = basic_authentication(headers).map_err(PublishError::AuthError)?;
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
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
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
