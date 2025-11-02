pub mod health;
pub mod subscriptions;
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
use serde::Serialize;
use std::error::Error;
use std::fmt::{self};
use subscriptions::*;
use tower::ServiceBuilder;
use tower_http::{ServiceBuilderExt, request_id::MakeRequestUuid, trace::TraceLayer};
use tracing::warn;

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

async fn handle_404(uri: Uri) -> impl IntoResponse {
    warn!("Route not found: {}", uri);
    (StatusCode::NOT_FOUND, "Not found")
}

#[derive(Serialize)]
pub struct ResponseMessage {
    message: String,
}

pub trait FormatterExt {
    fn write_error_chain(&mut self, e: &impl Error) -> fmt::Result;
}

impl FormatterExt for fmt::Formatter<'_> {
    fn write_error_chain(&mut self, e: &impl Error) -> fmt::Result {
        writeln!(self, "{e}")?;
        let mut cause = e.source();
        let mut depth = 1;

        while let Some(err) = cause {
            writeln!(self, "{:>width$}+ {err}", "", width = depth * 2)?;
            cause = err.source();
            depth += 1;
        }
        Ok(())
    }
}
