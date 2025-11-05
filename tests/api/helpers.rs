use reqwest::{StatusCode, Url};
use sqlx::types::Uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::Result;
use tokio::net::TcpListener;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use z2p::app_state::{create_email_client, init_tracing};
use z2p::configuration::get_configuration;
use z2p::{
    configuration::{AppState, DatabaseConfiguration},
    routes::get_router,
};

/// Only for integration tests.
#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
    pub api_client: reqwest::Client,
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: Url,
    pub plaintext: Url,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(format!("{}/subscribe", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// retrieve links from an email using `linkify`
    pub fn retrieve_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let confirmation_link = Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

            confirmation_link
        };

        let html = get_link(body["html"].as_str().unwrap());
        let plaintext = get_link(body["text"].as_str().unwrap());

        ConfirmationLinks { html, plaintext }
    }

    pub fn fake_body(&self) -> String {
        "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string()
    }

    pub async fn mock_mail_server(&self, status_code: StatusCode) {
        Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(status_code))
            .mount(&self.email_server)
            .await
    }

    /// Wait for an email request to be received, polling with retries.
    /// Returns the first received request when available.
    /// Returns an error if timeout is reached.
    pub async fn wait_for_email_request(&self) -> Result<wiremock::Request, String> {
        let timeout = std::time::Duration::from_secs(5);
        let poll_interval = std::time::Duration::from_millis(50);
        let start = std::time::Instant::now();

        // Initial small delay to allow background task to start
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        loop {
            if let Ok(requests) = self.email_server.received_requests().await {
                if !requests.is_empty() {
                    return Ok(requests[0].clone());
                }
            }

            if start.elapsed() >= timeout {
                tracing::warn!(
                    "Timeout waiting for email request after {:?}. \
                    Email may not have been sent in background task.",
                    timeout
                );
                return Err(format!(
                    "Timeout waiting for email request after {:?}",
                    timeout
                ));
            }

            tokio::time::sleep(poll_interval).await;
        }
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

/// It does the following:
/// 1. Tracing
/// 1. Mock Email Server
/// 1. Mutates configuration for test needs
/// 1. Spawns a tokio thread for running the axum server
pub async fn spawn_app_testing() -> Result<TestApp> {
    init_tracing()?;

    let email_server = MockServer::start().await;

    let config = {
        let mut c = get_configuration().expect("Failed to read Configuration");
        let email_client_url = reqwest::Url::parse(&email_server.uri())
            .map_err(|_| std::io::ErrorKind::InvalidInput)?;
        c.database.name = Uuid::new_v4().to_string();
        c.email_client.base_url = email_client_url;
        // randomized OS port
        c.application.port = 0;
        c
    };

    let base_url = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(&base_url).await?;

    let address = {
        let x = listener.local_addr()?;
        format!("http://{x}")
    };

    let db_pool = configure_test_database(&config.database).await;
    let email_client = create_email_client(&config);
    let api_client = reqwest::Client::new();

    let app_state = AppState {
        db_pool: db_pool.clone(),
        base_url: address.clone(),
        email_client,
    };

    let test_app = TestApp {
        address,
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
