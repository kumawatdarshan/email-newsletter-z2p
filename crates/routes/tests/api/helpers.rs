use argon2::{
    Argon2, Params,
    password_hash::{SaltString, rand_core::OsRng},
};
use reqwest::{StatusCode, Url};
use routes::get_router;
use settings::{DatabaseConfiguration, get_configuration};
use sqlx::{SqlitePool, migrate::Migrator, sqlite::SqlitePoolOptions, types::Uuid};
use state::{AppState, create_email_client};
use std::{io::Result, path::PathBuf};
use telemetry::init_tracing;
use tokio::net::TcpListener;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

/// Only for integration tests.
#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
    pub email_server: MockServer,
    pub api_client: reqwest::Client,
    pub test_user: TestUser,
}

#[derive(Debug)]
pub struct TestUser {
    pub user_id: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ConfirmationLinks {
    pub html: Url,
    pub plaintext: Url,
}

pub(crate) trait FakeData {
    fn fake_email(&self) -> String;
    fn fake_newsletter(&self) -> serde_json::Value;
    fn fake_invalid_account(&self) -> serde_json::Value;
}

impl TestApp {
    pub(crate) async fn post_newsletters(&self, body: serde_json::Value) -> reqwest::Response {
        self.post_newsletters_with_auth(body, &self.test_user.username, &self.test_user.password)
            .await
    }

    pub(crate) async fn post_newsletters_with_auth(
        &self,
        body: serde_json::Value,
        username: &str,
        password: &str,
    ) -> reqwest::Response {
        self.api_client
            .post(format!("{}/newsletters", &self.address))
            .json(&body)
            .basic_auth(username, Some(password))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub(crate) async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(format!("{}/subscribe", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    /// retrieve links from an email using `linkify`
    pub(crate) fn retrieve_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
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

    pub(crate) async fn mock_mail_server(&self, status_code: StatusCode) {
        Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(status_code))
            .mount(&self.email_server)
            .await
    }

    pub(crate) async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.api_client
            .post(format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to post login.")
    }

    pub(crate) async fn get_login_html(&self) -> String {
        self.api_client
            .get(format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute login html request")
            .text()
            .await
            .unwrap()
    }
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            user_id: Uuid::new_v4().to_string(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }
    async fn store(&self, pool: &SqlitePool) {
        use argon2::PasswordHasher;
        let salt = SaltString::generate(&mut OsRng);
        // from the dummy hash
        let pw_hash = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query!(
            r#"
            INSERT INTO users (user_id, username, password_hash)
            VALUES ($1, $2, $3)
        "#,
            self.user_id,
            self.username,
            pw_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to create test user");
    }
}

impl FakeData for TestApp {
    fn fake_email(&self) -> String {
        "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string()
    }

    fn fake_newsletter(&self) -> serde_json::Value {
        serde_json::json!({
           "title": "Newsletter Title",
           "content": {
               "text": "Plain-text Body",
               "html": "<p>HTML body</p>",
           }
        })
    }

    fn fake_invalid_account(&self) -> serde_json::Value {
        serde_json::json!({
            "username": "random-username",
            "password": "random-password",
        })
    }
}

/// Creating a uuid named db through `PgConnection` and then doing the migrations through `PgPool`
async fn configure_test_database(settings: &DatabaseConfiguration) -> SqlitePool {
    let connection_pool = SqlitePoolOptions::new()
        .connect_with(settings.options().shared_cache(true))
        .await
        .unwrap();

    let migrations_dir = PathBuf::from(concat!(env!("CARGO_WORKSPACE_DIR"), "/migrations"));

    let migrator = Migrator::new(migrations_dir)
        .await
        .expect("Failed to load migrations");

    migrator
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
        c.database.url = "sqlite::memory:".to_owned();
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

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let test_user = TestUser::generate();
    test_user.store(&db_pool).await;

    let app_state = AppState {
        db_pool: db_pool.clone(),
        base_url: address.clone(),
        email_client,
        hmac_secret: config.application.hmac_secret.into(),
    };

    let test_app = TestApp {
        address,
        db_pool,
        email_server,
        api_client,
        test_user,
    };

    let router = get_router(app_state);

    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Test server failed");
    });

    Ok(test_app)
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
