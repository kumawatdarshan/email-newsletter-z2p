mod health;
mod home;
mod login;
mod middlewares;
mod newsletters;
mod routes;
mod startup;
mod subscriptions;
mod subscriptions_confirm;
use email_client::EmailClient;
use home::*;

use axum::{
    extract::FromRef,
    http::{StatusCode, Uri},
};
use repository::Repository;
use serde::Serialize;
use tracing::warn;

// re-exports
pub use routes::routes_path;
pub use startup::{Application, ApplicationBuilder};

/// State needed for various services like ~psql~,sqlite, redis, etc
#[derive(Debug, Clone, FromRef)]
pub struct AppState {
    pub repo: Repository,
    pub email_client: EmailClient,
    pub base_url: String,
}

// Only for debugging. Should be removed in production to declutter the logs.
async fn handle_404(uri: Uri) -> StatusCode {
    warn!("Route not found: {}", uri);
    StatusCode::NOT_FOUND
}

#[derive(Serialize)]
pub(crate) struct ResponseMessage {
    message: String,
}
