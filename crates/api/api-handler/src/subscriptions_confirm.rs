use anyhow::Context;
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::{Repository, subscriptions_confirm::SubscriptionsConfirmRepository};
use serde::Deserialize;

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

#[tracing::instrument(name = "Confirm a pending subscriber", skip(repo, parameters))]
pub(crate) async fn confirm(
    State(repo): State<Repository>,
    Query(parameters): Query<Parameters>,
) -> Result<impl IntoResponse, ConfirmationError> {
    let subscriber_id = repo
        .get_subscriber_id_from_token(&parameters.subscription_token)
        .await
        .context("Failed to retrieve subscriber ID from token")?
        .ok_or(ConfirmationError::InvalidToken)?;

    let was_updated = repo
        .confirm_subscriber(subscriber_id)
        .await
        .context("Failed to mark subscriber as confirmed")?;

    let message = if was_updated {
        "Subscription confirmed successfully!".to_string()
    } else {
        "Your subscription was already confirmed.".to_string()
    };

    Ok((StatusCode::OK, Json(ResponseMessage { message })))
}
