use std::sync::OnceLock;

use crate::{
    configuration::{Configuration, get_configuration},
    email_client::EmailClient,
    telemetry::get_subscriber,
};
use tokio::net::TcpListener;
use tracing::subscriber::set_global_default;

static TRACING: OnceLock<()> = OnceLock::new();

#[derive(Debug)]
pub struct AppFactory {
    pub config: Configuration,
    pub test_mode: bool,
}

impl AppFactory {
    pub fn new(test_mode: bool) -> std::io::Result<Self> {
        let config = get_configuration().expect("Failed to read Configuration");

        Ok(Self { config, test_mode })
    }

    pub fn init_subscriber(self) -> Result<Self, std::io::Error> {
        let subscriber = get_subscriber("z2p".into(), "debug".into(), std::io::stdout)?;
        TRACING.get_or_init(|| {
            tracing_log::LogTracer::init().expect("Failed to set logger.");
            set_global_default(subscriber).expect("Failed to set tracing-subscriber.");
        });
        Ok(self)
    }

    pub async fn create_listener(&self) -> std::io::Result<TcpListener> {
        let test = "127.0.0.1:0".to_string();
        let prod = format!(
            "{}:{}",
            self.config.application.host, self.config.application.port
        );

        let bind_addr = if self.test_mode { test } else { prod };
        TcpListener::bind(bind_addr).await
    }

    pub fn create_email_client(&self) -> EmailClient {
        let sender_email = self
            .config
            .email_client
            .sender()
            .expect("Invalid Sender email");

        let timeout = self.config.email_client.timeout();

        EmailClient::new(
            self.config.email_client.base_url.clone(),
            sender_email,
            self.config.email_client.authorization_token.clone(),
            timeout,
        )
    }
}
