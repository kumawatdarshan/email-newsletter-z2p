use crate::{
    handle_404,
    health::health_check,
    home,
    login::{login, login_form},
    newsletters::publish_newsletter,
    subscriptions::subscribe,
    subscriptions_confirm::confirm,
};
use axum::{
    Router,
    routing::{get, post},
};
use axum_messages::MessagesManagerLayer;
use state::AppState;
use telemetry::RequestIdMakeSpan;
use tower::ServiceBuilder;
use tower_http::{ServiceBuilderExt, request_id::MakeRequestUuid, trace::TraceLayer};

pub fn get_router(app_state: AppState) -> Router {
    let request_id_middleware = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan))
        .propagate_x_request_id();

    let is_dev_server = true;
    let session_store = tower_sessions::MemoryStore::default();
    let session_layer =
        tower_sessions::SessionManagerLayer::new(session_store).with_secure(!is_dev_server);

    let session_middleware = ServiceBuilder::new()
        .layer(MessagesManagerLayer)
        .layer(session_layer);

    Router::new()
        .route("/", get(home))
        .route("/login", post(login).get(login_form))
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
        .route("/subscribe/confirm", get(confirm))
        .route("/newsletters", post(publish_newsletter))
        // unlike in `actix_session` implementation, we don't need to provide any signing key because cookie has no session data.
        // https://github.com/maxcountryman/tower-sessions/discussions/100
        // > tower-sessions doesn't provide signing because no data is stored in the cookie.
        // > In other words, the cookie value is a pointer to the data stored server side.
        .layer(session_middleware)
        .layer(request_id_middleware)
        .fallback(handle_404)
        .with_state(app_state)
}
