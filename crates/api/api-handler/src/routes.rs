use crate::{
    AppState, handle_404,
    health::health_check,
    home,
    login::{login, login_form},
    middlewares::{RequestIdMakeSpan, authentication::require_authentication},
    newsletters::publish_newsletter,
    subscriptions::subscribe_to_newsletter,
    subscriptions_confirm::subscriptions_confirm,
};
use axum::{Router, middleware};
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

    let protected_routes = Router::new()
        .typed_post(publish_newsletter)
        // Add other protected routes here
        .route_layer(middleware::from_fn_with_state(
            app_state.repo.clone(), // Extract repo from app_state
            require_authentication,
        ));

    let public_routes = Router::new()
        .typed_get(home)
        .typed_post(login)
        .typed_get(login_form)
        .typed_get(health_check)
        .typed_post(subscribe_to_newsletter)
        .typed_get(subscriptions_confirm);

    let router = Router::new()
        .merge(protected_routes)
        .merge(public_routes)
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
}
