use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use newsletter_macros::{DebugChain, IntoErrorResponse};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::ResponseMessage;

#[derive(Debug, Deserialize)]
pub(crate) struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum ConfirmationError {
    #[error("The confirmation token is invalid or has expired")]
    #[status(StatusCode::UNAUTHORIZED)]
    InvalidToken,

    #[error("Something went wrong")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(db_pool, parameters))]
pub(crate) async fn confirm(
    State(db_pool): State<SqlitePool>,
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse, ConfirmationError> {
    let subscriber_id = get_subscriber_id_from_token(&db_pool, &parameters.subscription_token)
        .await
        .context("Failed to retrieve subscriber ID from token")?
        .ok_or(ConfirmationError::InvalidToken)?;

    let was_updated = confirm_subscriber(&db_pool, subscriber_id)
        .await
        .context("Failed to mark subscriber as confirmed")?;

    let message = if was_updated {
        "Subscription confirmed successfully!".to_string()
    } else {
        "Your subscription was already confirmed.".to_string()
    };

    Ok((StatusCode::OK, Json(ResponseMessage { message })))
}

#[tracing::instrument(name = "Mark subscriber as Confirmed", skip(subscriber_id, pool))]
async fn confirm_subscriber(pool: &SqlitePool, subscriber_id: String) -> Result<bool, sqlx::Error> {
    // TODO: ADD TIMESTAMP in schema TO AUTOMATICALLY invalidate token after 24h
    let result = sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed'
           WHERE id = $1 AND status = 'pending_confirmation'"#,
        subscriber_id
    )
    .execute(pool)
    .await?;

    let was_updated = result.rows_affected() > 0;

    let message = if was_updated {
        "Successfully confirmed new subscriber"
    } else {
        "Subscriber already confirmed (idempotent operation)"
    };

    tracing::info!(
        %subscriber_id,
        message
    );

    Ok(was_updated)
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(pool, subscription_token))]
async fn get_subscriber_id_from_token(
    pool: &SqlitePool,
    subscription_token: &str,
) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens
           WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}
