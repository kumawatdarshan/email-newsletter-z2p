use sqlx::types::Uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::Result;
use tokio::net::TcpListener;
use wiremock::MockServer;
use z2p::app_state::{create_email_client, init_tracing};
use z2p::configuration::get_configuration;
use z2p::{
    configuration::{AppState, DatabaseConfiguration},
    routes::get_router,
};

/// Only for integration tests.
#[derive(Debug)]
pub struct TestApp {
    pub addr: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub api_client: reqwest::Client,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(format!("{}/subscribe", &self.addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

/// Creating a uuid named db through `PgConnection` and then doing the migrations through `PgPool`
async fn configure_test_database(settings: &DatabaseConfiguration) -> PgPool {
    let mut connection = PgConnection::connect_with(&settings.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(format!(r#"CREATE DATABASE "{}""#, settings.name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(settings.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub async fn spawn_app_testing() -> Result<TestApp> {
    init_tracing()?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let email_server = MockServer::start().await;

    let config = {
        let mut c = get_configuration().expect("Failed to read Configuration");
        let email_client_url = reqwest::Url::parse(&email_server.uri())
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        c.database.name = Uuid::new_v4().to_string();
        c.email_client.base_url = email_client_url;
        c
    };

    let db_pool = configure_test_database(&config.database).await;

    let email_client = create_email_client(&config);

    let app_state = AppState {
        db_pool: db_pool.clone(),
        email_client,
    };

    let api_client = reqwest::Client::new();

    let test_app = TestApp {
        addr: format!("http://{}", listener.local_addr()?),
        db_pool,
        email_server,
        api_client,
    };

    let router = get_router(app_state);

    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Test server failed");
    });

    Ok(test_app)
}
