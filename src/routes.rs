pub mod health;
pub mod subscription;
use crate::configuration::AppState;
use crate::telemetry::RequestIdMakeSpan;
use axum::{
    Router,
    routing::{get, post},
};
use health::*;
use subscription::*;
use tower::ServiceBuilder;
use tower_http::{ServiceBuilderExt, request_id::MakeRequestUuid, trace::TraceLayer};

pub fn get_router(app_state: AppState) -> Router {
    let middlewares = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan))
        .propagate_x_request_id();

    Router::new()
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
        .layer(middlewares)
        .with_state(app_state.into())
}
