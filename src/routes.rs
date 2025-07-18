pub mod health;
pub mod subscribe;

use axum::{
    Router,
    routing::{get, post},
};

use health::health_check;
use subscribe::subscribe;

pub fn routes() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
}
