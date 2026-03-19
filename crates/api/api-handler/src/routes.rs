use crate::{
    AppState,
    admin::{
        dashboard::admin_dashboard,
        newsletters::{newsletter_issue_form, publish_newsletter},
        password::{change_password, password_change_form},
    },
    handle_404,
    health::health_check,
    home,
    login::{login, login_form},
    middlewares::RequestIdMakeSpan,
    signup::signup,
    subscriptions::subscribe_to_newsletter,
    subscriptions_confirm::subscriptions_confirm,
};
use axum::{
    Router,
    routing::{get, post},
};
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

    use routes_path::*;

    let admin_routes = Router::new()
        .route(
            ADMIN_NEWSLETTERS,
            get(newsletter_issue_form).post(publish_newsletter),
        )
        .route(ADMIN_DASHBOARD, get(admin_dashboard))
        .route(
            ADMIN_PASSWORD,
            get(password_change_form).post(change_password),
        );

    let subscription_routes = Router::new()
        .route(SUBSCRIPTIONS, post(subscribe_to_newsletter))
        .route(SUBSCRIPTIONS_CONFIRM, get(subscriptions_confirm));

    // Authentication is handled via type system, specifically axum's extractor
    // `RequireAuth` for browser consumer and redirects to `/login` while
    // `AuthenticatedUser` for api consumer and returns 401
    let router = Router::new()
        .route(INDEX, get(home))
        .route(HEALTH_CHECK, get(health_check))
        .route(LOGIN, get(login_form).post(login))
        .route(SIGN_UP, post(signup))
        .merge(admin_routes)
        .merge(subscription_routes)
        .layer(MessagesManagerLayer)
        .layer(session_layer)
        .layer(request_id_middleware)
        .fallback(handle_404)
        .with_state(app_state);

    Ok(router)
}

pub mod routes_path {
    pub const INDEX: &str = "/";
    pub const HEALTH_CHECK: &str = "/health";
    pub const LOGIN: &str = "/login";
    pub const SIGN_UP: &str = "/signup";

    pub const SUBSCRIPTIONS: &str = "/subscriptions";
    pub const SUBSCRIPTIONS_CONFIRM: &str = "/subscriptions/confirm";

    pub const ADMIN: &str = "/admin/dashboard";
    pub const ADMIN_DASHBOARD: &str = "/admin/dashboard";
    pub const ADMIN_PASSWORD: &str = "/admin/password";
    pub const ADMIN_NEWSLETTERS: &str = "/admin/newsletters";
}
