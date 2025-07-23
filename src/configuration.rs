use secrecy::{ExposeSecret, SecretString};

use config::{Config, ConfigError, File};
use serde::Deserialize;
use sqlx::{PgPool, postgres::PgConnectOptions};

pub type Port = u16;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: Port,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    pub port: Port,
    pub db_name: String,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.db_name)
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;

    settings.try_deserialize::<Settings>()
}

/// State needed for various services like psql, redis, etc
#[derive(Clone, Debug)]
pub struct AppState {
    pub db_pool: PgPool,
}
