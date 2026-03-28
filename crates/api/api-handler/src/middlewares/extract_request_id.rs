use axum::extract::Request;
use tower_http::trace::MakeSpan;
use tracing::Span;

#[derive(Clone, Debug)]
pub struct RequestIdMakeSpan;

pub const REQUEST_ID: &str = "x-request-id";

impl<B> MakeSpan<B> for RequestIdMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get(REQUEST_ID)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        tracing::info_span!(
            "request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            "x-request-id" = %request_id,
        )
    }
}
