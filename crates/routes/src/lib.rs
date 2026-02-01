mod authentication;
mod health;
mod home;
mod login;
mod newsletters;
mod routes;
mod subscriptions;
mod subscriptions_confirm;

use home::*;

use axum::http::{StatusCode, Uri};
use serde::Serialize;
use tracing::warn;

// re-exports
pub use routes::get_router;

// Only for debugging. Should be removed in production to declutter the logs.
async fn handle_404(uri: Uri) -> StatusCode {
    warn!("Route not found: {}", uri);
    StatusCode::NOT_FOUND
}

#[derive(Serialize)]
pub(crate) struct ResponseMessage {
    message: String,
}
