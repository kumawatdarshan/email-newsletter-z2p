use sqlx::PgPool;
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::{
    configuration::{AppState, Configuration, get_configuration},
    email_client::EmailClient,
    startup::configure_test_database,
    telemetry::{get_subscriber, init_subscriber},
};

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
        init_subscriber(subscriber)?;
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

    pub async fn create_db_pool(&mut self) -> PgPool {
        if self.test_mode {
            self.config.database.name = Uuid::new_v4().to_string();
            configure_test_database(&self.config.database).await
        } else {
            PgPool::connect_lazy_with(self.config.database.with_db())
        }
    }

    pub fn create_email_client(&self) -> EmailClient {
        let sender_email = self
            .config
            .email_client
            .sender()
            .expect("Invalid Sender email");
        EmailClient::new(self.config.email_client.base_url.clone(), sender_email)
    }
}
