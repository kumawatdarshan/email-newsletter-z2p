use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use serde::Deserialize;
use sqlx::SqlitePool;
use state::AppState;

use crate::ResponseMessage;
use crate::error::write_error_chain;

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
    email: String,
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
) -> impl IntoResponse {
    StatusCode::OK
}

#[tracing::instrument(name = "Get Confirmed Subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &SqlitePool,
) -> Result<Vec<ConfirmedSubscriber>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ConfirmedSubscriber,
        r#"
            SELECT email FROM subscriptions WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
