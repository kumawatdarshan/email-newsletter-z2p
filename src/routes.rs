pub mod health;
pub mod subscribe;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

pub use health::*;
pub use subscribe::*;

use crate::configuration::AppState;

pub fn get_router(connection: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(connection)
}
