use crate::{
    handle_404, health::health_check, newsletters::publish_newsletter, subscriptions::subscribe,
    subscriptions_confirm::confirm,
};
use axum::{
    Router,
    routing::{get, post},
};
use state::AppState;
use telemetry::RequestIdMakeSpan;
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
        .route("/subscribe/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        .layer(middlewares)
        .fallback(handle_404)
        .with_state(app_state.into())
}
