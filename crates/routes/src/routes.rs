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
use settings::Port;
use state::AppState;
use telemetry::RequestIdMakeSpan;
use tower::ServiceBuilder;
use tower_http::{ServiceBuilderExt, request_id::MakeRequestUuid, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};
use tower_sessions_redis_store::{RedisStore, fred::prelude::Pool};

pub async fn get_router(app_state: AppState, redis_pool: Pool) -> anyhow::Result<Router> {
    let request_id_middleware = ServiceBuilder::new()
        .set_x_request_id(MakeRequestUuid)
        .layer(TraceLayer::new_for_http().make_span_with(RequestIdMakeSpan))
        .propagate_x_request_id();

    let session_store = RedisStore::new(redis_pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

    let router = Router::new()
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
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .layer(request_id_middleware)
        .fallback(handle_404)
        .with_state(app_state);

    Ok(router)
}
