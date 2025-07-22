use config::{Config, ConfigError, File};
use serde::Deserialize;
use sqlx::PgPool;

pub type Port = u16;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: Port,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: Port,
    pub db_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.db_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_configuration() -> Result<Settings, ConfigError> {
    let settings = Config::builder()
        .add_source(File::new("configuration.yaml", config::FileFormat::Yaml))
        .build()?;

    settings.try_deserialize::<Settings>()
}

/// State needed for various services like psql, redis, etc
pub struct AppState {
    pub db_pool: PgPool,
}
