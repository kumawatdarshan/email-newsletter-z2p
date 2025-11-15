use crate::{FormatterExt, ResponseMessage};
use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::{PgPool, types::Uuid};
use state::AppState;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub(crate) struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmationError {
    #[error("The confirmation token is invalid or has expired")]
    InvalidToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ConfirmationError {
    fn into_response(self) -> Response {
        tracing::error!(exception.details = ?self, exception.message = %self);
        let status_code = match self {
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = self.to_string();

        (status_code, Json(ResponseMessage { message })).into_response()
    }
}

impl std::fmt::Debug for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_error_chain(self)
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub(crate) async fn confirm(
    State(state): State<Arc<AppState>>,
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse, ConfirmationError> {
    let subscriber_id =
        get_subscriber_id_from_token(&state.db_pool, &parameters.subscription_token)
            .await
            .context("Failed to retrieve subscriber ID from token")?
            .ok_or(ConfirmationError::InvalidToken)?;

    let was_updated = confirm_subscriber(&state.db_pool, subscriber_id)
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
async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<bool, sqlx::Error> {
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

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens
           WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}
