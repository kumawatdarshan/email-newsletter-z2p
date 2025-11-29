pub(crate) mod health;
pub(crate) mod newsletters;
pub(crate) mod routes;
pub(crate) mod subscriptions;
pub(crate) mod subscriptions_confirm;

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
