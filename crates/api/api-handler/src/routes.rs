use crate::{
    AppState,
    admin::admin_dashboard,
    handle_404,
    health::health_check,
    home,
    login::{login, login_form},
    middlewares::RequestIdMakeSpan,
    newsletters::publish_newsletter,
    subscriptions::subscribe_to_newsletter,
    subscriptions_confirm::subscriptions_confirm,
};
use axum::Router;
use axum_extra::routing::RouterExt;
use axum_messages::MessagesManagerLayer;
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
        .typed_post(publish_newsletter)
        .typed_get(admin_dashboard)
        .typed_post(login)
        .typed_get(home)
        .typed_get(login_form)
        .typed_get(health_check)
        .typed_post(subscribe_to_newsletter)
        .typed_get(subscriptions_confirm)
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .layer(request_id_middleware)
        .fallback(handle_404)
        .with_state(app_state);

    Ok(router)
}

pub mod routes_path {
    use axum_extra::routing::TypedPath;
    use serde::Deserialize;

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/")]
    pub struct Index;

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/health")]
    pub struct HealthCheck;

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/subscriptions")]
    pub struct Subscriptions;

    #[derive(TypedPath, Deserialize)]
    #[typed_path("/subscriptions/confirm")]
    pub struct SubscriptionsConfirm;

    #[derive(Deserialize, TypedPath)]
    #[typed_path("/login")]
    pub struct Login;

    #[derive(Deserialize, TypedPath)]
    #[typed_path("/newsletters")]
    pub struct Newsletters;

    #[derive(Deserialize, TypedPath)]
    #[typed_path("/admin/dashboard")]
    pub struct AdminDashboard;
}
