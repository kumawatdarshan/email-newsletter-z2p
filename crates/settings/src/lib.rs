use config::{Config, ConfigError, File};
use domain::SubscriberEmail;
use secrecy::SecretString;
use serde::Deserialize;
use sqlx::ConnectOptions;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use std::path::PathBuf;
use std::str::FromStr;

pub fn get_configuration() -> Result<Configuration, ConfigError> {
    dotenvy::dotenv().ok();

    // this can be compile time because we are providing from the .cargo/config.toml
    let configuration_dir = PathBuf::from(concat!(env!("CARGO_WORKSPACE_DIR"), "/configuration"));

    // this can't be as it can be changed in runtime
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or("local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT variable");

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
    pub redis: RedisConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct RedisConfiguration {
    pub port: Port,
    pub host: SecretString,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationConfiguration {
    pub port: Port,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfiguration {
    pub url: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct EmailClientConfiguration {
    pub base_url: url::Url,
    pub sender_email: String,
    pub authorization_token: SecretString,
    pub timeout_ms: u64,
}

impl EmailClientConfiguration {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms)
    }
}

impl DatabaseConfiguration {
    pub fn options(&self) -> SqliteConnectOptions {
        SqliteConnectOptions::from_str(&self.url)
            .expect("Invalid SQLite path")
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
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
