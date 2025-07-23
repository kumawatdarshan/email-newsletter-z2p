pub mod health;
pub mod subscribe;

use crate::configuration::AppState;
use axum::{
    Router,
    http::Request,
    routing::{get, post},
};
use health::*;
use subscribe::*;
use tower::ServiceBuilder;
use tower_http::{
    ServiceBuilderExt,
    request_id::{MakeRequestId, RequestId},
    trace::TraceLayer,
};
use uuid::Uuid;

// Define your ID generator
#[derive(Clone)]
struct MyMakeRequestId;
impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        Some(RequestId::new(Uuid::new_v4().to_string().parse().unwrap()))
    }
}

pub fn get_router(connection: AppState) -> Router {
    let middlewares = ServiceBuilder::new()
        .set_x_request_id(MyMakeRequestId)
        .layer(TraceLayer::new_for_http())
        .propagate_x_request_id();

    Router::new()
        .route("/health", get(health_check))
        .route("/subscribe", post(subscribe))
        .layer(middlewares)
        .with_state(connection.into())
}
