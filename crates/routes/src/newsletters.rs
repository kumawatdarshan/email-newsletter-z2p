use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
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

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_error_chain(f, self)
    }
}

impl IntoResponse for PublishError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!(exception.details = ?self, exception.message = %self);

        let status_code = match &self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let message = self.to_string();

        (status_code, Json(ResponseMessage { message })).into_response()
    }
}

#[axum::debug_handler]
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
