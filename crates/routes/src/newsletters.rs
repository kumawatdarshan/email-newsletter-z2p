use std::sync::Arc;

use anyhow::{Context, anyhow};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use domain::SubscriberEmail;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use serde::Deserialize;
use sqlx::SqlitePool;
use state::AppState;

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
    #[error(transparent)]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

pub(crate) async fn publish_newsletter(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
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
