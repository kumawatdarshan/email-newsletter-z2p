use std::sync::OnceLock;
use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, fmt::MakeWriter, layer::SubscriberExt};

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

static TRACING: OnceLock<()> = OnceLock::new();

pub fn init_tracing() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "debug".into(), std::io::stdout)?;
    TRACING.get_or_init(|| {
        tracing_log::LogTracer::init().expect("Failed to set logger.");
        set_global_default(subscriber).expect("Failed to set tracing-subscriber.");
    });
    Ok(())
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
