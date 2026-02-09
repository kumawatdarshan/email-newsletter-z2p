use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::subscriptions_confirm::SubscriptionsConfirmRepository;
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
    let subscriber_id = SubscriptionsConfirmRepository::get_subscriber_id_from_token(
        &db_pool,
        &parameters.subscription_token,
    )
    .await
    .context("Failed to retrieve subscriber ID from token")?
    .ok_or(ConfirmationError::InvalidToken)?;

    let was_updated = SubscriptionsConfirmRepository::confirm_subscriber(&db_pool, subscriber_id)
        .await
        .context("Failed to mark subscriber as confirmed")?;

    let message = if was_updated {
        "Subscription confirmed successfully!".to_string()
    } else {
        "Your subscription was already confirmed.".to_string()
    };

    Ok((StatusCode::OK, Json(ResponseMessage { message })))
}
