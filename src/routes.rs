pub mod health;
pub mod subscription;
pub mod subscriptions_confirm;

use crate::telemetry::RequestIdMakeSpan;
use crate::{configuration::AppState, routes::subscriptions_confirm::confirm};
use axum::http::{StatusCode, Uri};
use axum::response::IntoResponse;
use axum::{
    Router,
    routing::{get, post},
};
use health::*;
use subscription::*;
use tower::ServiceBuilder;
use tower_http::{ServiceBuilderExt, request_id::MakeRequestUuid, trace::TraceLayer};
use tracing::warn;

async fn handle_404(uri: Uri) -> impl IntoResponse {
    warn!("Route not found: {}", uri);
    (StatusCode::NOT_FOUND, "Not found")
}

pub fn get_router(app_state: AppState) -> Router {
    let middlewares = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan))
        .propagate_x_request_id();

    Router::new()
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
        .route("/subscribe/confirm", get(confirm))
        .layer(middlewares)
        .fallback(handle_404)
        .with_state(app_state.into())
}
