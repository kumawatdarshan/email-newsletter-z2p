pub mod health;
pub mod subscribe;

use axum::{
    Router,
    routing::{get, post},
};

pub use health::*;
pub use subscribe::*;

pub fn router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
}
