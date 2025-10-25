use axum::http::Request;
use tower_http::trace::MakeSpan;
use tracing::Span;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

#[derive(Clone, Debug)]
pub struct RequestIdMakeSpan;

impl<B> MakeSpan<B> for RequestIdMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get("x-request-id")
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

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> Result<impl Subscriber + Send + Sync, std::io::Error>
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(env_filter.into());
    let formatting_layer = BunyanFormattingLayer::new(name, sink);

    Ok(Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer))
}
