use crate::authentication::{basic_authentication, validate_credentials};
use anyhow::{Context, anyhow};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use domain::SubscriberEmail;
use newsletter_macros::{DebugChain, IntoErrorResponse};
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
    // this will delegate error to the super::authentication::AuthError
    #[error("Authentication failed.")]
    #[status(StatusCode::UNAUTHORIZED)]
    #[headers([header::WWW_AUTHENTICATE = r#"Basic realm="publish""#])]
    AuthError(#[from] super::authentication::AuthError),

    #[error("Something went wrong.")]
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
    let credentials = basic_authentication(headers)?;

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(credentials, &state.db_pool).await?;
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
            // By this I mean, separate warning logs for each invalid subscriber
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
