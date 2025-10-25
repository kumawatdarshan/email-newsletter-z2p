use std::sync::OnceLock;

use crate::{
    configuration::{Configuration, get_configuration},
    email_client::EmailClient,
    telemetry::get_subscriber,
};
use tokio::net::TcpListener;
use tracing::subscriber::set_global_default;

static TRACING: OnceLock<()> = OnceLock::new();

pub fn init_tracing() -> std::io::Result<()> {
    let subscriber = get_subscriber("z2p".into(), "debug".into(), std::io::stdout)?;
    TRACING.get_or_init(|| {
        tracing_log::LogTracer::init().expect("Failed to set logger.");
        set_global_default(subscriber).expect("Failed to set tracing-subscriber.");
    });
    Ok(())
}

pub async fn create_listener(config: Configuration) -> std::io::Result<TcpListener> {
    let bind_addr = format!("{}:{}", config.application.host, config.application.port);
    TcpListener::bind(bind_addr).await
}

pub fn create_email_client(config: &Configuration) -> EmailClient {
    let sender_email = config.email_client.sender().expect("Invalid Sender email");
    let timeout = config.email_client.timeout();

    EmailClient::new(
        config.email_client.base_url.clone(),
        sender_email,
        config.email_client.authorization_token.clone(),
        timeout,
    )
}
