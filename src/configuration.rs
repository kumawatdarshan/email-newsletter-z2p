use config::Config;
use config::{ConfigError, File};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use sqlx::postgres::PgSslMode;
use sqlx::{ConnectOptions, PgPool, postgres::PgConnectOptions};

use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;

pub fn get_configuration() -> Result<Configuration, ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_dir = base_path.join("configuration");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("Faild to parse APP_ENVIRONMENT variable");

    // this would set APP_{Configuration}_{Field}
    let settings = Config::builder()
        .add_source(File::from(configuration_dir.join("base.json")))
        .add_source(File::from(
            configuration_dir.join(format!("{}.json", environment.as_str())),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Configuration>()
}

pub type Port = u16;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
    pub application: ApplicationConfiguration,
    pub email_client: EmailClientConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationConfiguration {
    pub port: Port,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfiguration {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    pub port: Port,
    pub name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize, Debug)]
pub struct EmailClientConfiguration {
    pub base_url: String,
    pub sender_email: String,
}

impl EmailClientConfiguration {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

impl DatabaseConfiguration {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.name)
            .log_statements(tracing_log::log::LevelFilter::Trace)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `local` or `production`."
            )),
        }
    }
}

/// State needed for various services like psql, redis, etc
#[derive(Debug)]
pub struct AppState {
    pub db_pool: PgPool,
    pub email_client: EmailClient,
}
