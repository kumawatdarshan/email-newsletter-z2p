use anyhow::Context;
use api_handler::{
    ApplicationBuilder,
    routes_path::{LOGIN, SUBSCRIPTIONS},
};
use argon2::{
    Argon2, Params,
    password_hash::{SaltString, rand_core::OsRng},
};
use configuration::{DatabaseConfiguration, get_configuration};
use repository::Repository;
use reqwest::{RequestBuilder, Response, StatusCode, Url};
use sqlx::{SqlitePool, migrate::Migrator, sqlite::SqlitePoolOptions, types::Uuid};
use std::{collections::HashMap, path::PathBuf};
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

#[derive(Debug, Clone)]
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

impl TestApp {
    pub(crate) fn typed_path(&self, path: &str) -> Url {
        let base_url = Url::parse(&self.address)
            .unwrap_or_else(|err| panic!("Failed to parse base address: {}\n{err}", self.address));

        base_url.join(path).expect("Failed to join path")
    }

    pub(crate) async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        self.api_client
            .post(self.typed_path(SUBSCRIPTIONS))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub trait AuthenticatedRequest: Sized {
    /// Attaches HTTP Basic Auth credentials from a `TestUser`.
    /// use for Auth failure
    async fn login(&self, user: &TestUser) -> anyhow::Result<&Self>;
}

impl AuthenticatedRequest for TestApp {
    async fn login(&self, user: &TestUser) -> anyhow::Result<&Self> {
        let form = [("username", &user.username), ("password", &user.password)];

        let res = self.post(LOGIN).form(&form).send().await?;

        assert!(
            res.status().is_success() || res.status().is_redirection(),
            "Login failed: {}",
            res.status()
        );

        Ok(self)
    }
}

pub trait TestAppRequests {
    fn get(&self, path: &str) -> RequestBuilder;
    fn post(&self, path: &str) -> RequestBuilder;
    fn typed_url(&self, path: &str) -> Url;
}

impl TestAppRequests for TestApp {
    fn typed_url(&self, route: &str) -> Url {
        let base = Url::parse(&self.address)
            .unwrap_or_else(|e| panic!("Failed to parse base address: {}\n{e}", self.address));
        base.join(route).expect("Failed to join path")
    }

    fn get(&self, route: &str) -> RequestBuilder {
        self.api_client.get(self.typed_url(route))
    }

    fn post(&self, route: &str) -> RequestBuilder {
        self.api_client.post(self.typed_url(route))
    }
}

pub trait ResponseAssertions {
    /// Assert the response is a 303 redirect to `location`.
    fn assert_redirect_to(self, location: &str) -> Self;

    /// Assert an exact status code.
    fn assert_status(self, expected: StatusCode) -> Self;

    /// Assert an exact status code.
    fn assert_status_with_msg(self, expected: StatusCode, msg: &str) -> Self;
}

impl ResponseAssertions for Response {
    fn assert_redirect_to(self, location: &str) -> Self {
        assert_eq!(
            StatusCode::SEE_OTHER,
            self.status(),
            "Expected 303 SEE_OTHER redirect"
        );
        assert_eq!(
            location,
            self.headers()
                .get("Location")
                .expect("Missing Location header"),
            "Redirect target mismatch"
        );
        self
    }

    fn assert_status(self, expected: StatusCode) -> Self {
        assert_eq!(expected, self.status());
        self
    }

    fn assert_status_with_msg(self, expected: StatusCode, msg: &str) -> Self {
        assert_eq!(expected, self.status(), "{msg}");
        self
    }
}

pub(crate) trait FakeData {
    fn fake_email(&self) -> String;
    fn fake_newsletter(&self) -> serde_json::Value;
    fn fake_invalid_account(&self) -> serde_json::Value;
}

impl FakeData for TestApp {
    fn fake_email(&self) -> String {
        "name=le%20guin&email=ursula_le_guin%40gmail.com".to_string()
    }

    fn fake_newsletter(&self) -> serde_json::Value {
        serde_json::json!({
           "title": "Newsletter Title",
           "text": "Plain-text Body",
           "html": "<p>HTML body</p>",
           "idempotency_key": Uuid::new_v4().to_string()
        })
    }

    fn fake_invalid_account(&self) -> serde_json::Value {
        serde_json::json!({
            "username": "random-username",
            "password": "random-password",
        })
    }
}

impl TestApp {
    /// Mounts a one-shot mock on the fake email server.
    pub(crate) async fn mock_mail_server(&self, status_code: StatusCode) {
        Mock::given(path("/email"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(status_code))
            .mount(&self.email_server)
            .await
    }

    /// retrieve links from an email using `linkify`
    pub(crate) fn retrieve_links(
        &self,
        email_request: &wiremock::Request,
    ) -> anyhow::Result<ConfirmationLinks> {
        let body: HashMap<String, String> =
            serde_urlencoded::from_bytes(&email_request.body).context("Failed to parse form")?;

        fn get_link(s: &str) -> anyhow::Result<Url> {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();

            assert_eq!(links.len(), 1);

            let raw_link = links[0].as_str().to_owned();
            let link = Url::parse(&raw_link).context("Failed to parse url")?;

            assert_eq!(link.host_str().unwrap(), "127.0.0.1");

            Ok(link)
        }

        Ok(ConfirmationLinks {
            html: get_link(body["html"].as_str())?,
            plaintext: get_link(body["text"].as_str())?,
        })
    }
}

impl TestUser {
    pub fn new() -> Self {
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
pub async fn spawn_app_testing() -> anyhow::Result<TestApp> {
    telemetry::init_tracing().context("Failed to initialize tracing.")?;

    let email_server = MockServer::start().await;

    let api_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let config = {
        let mut c = get_configuration().expect("Failed to read Configuration");
        let email_client_url =
            url::Url::parse(&email_server.uri()).context("Invalid Input for email client")?;
        c.database.url = "sqlite::memory:".to_owned();
        c.email_client.base_url = email_client_url;
        // randomized OS port
        c.application.port = 0;
        c
    };

    let db_pool = configure_test_database(&config.database).await;
    let test_user = TestUser::new();
    test_user.store(&db_pool).await;

    let app = ApplicationBuilder::new(&config)
        .with_db_pool(Repository::new(db_pool.clone()))
        .build()
        .await?;

    let test_app = TestApp {
        address: app.address(),
        db_pool,
        email_server,
        api_client,
        test_user,
    };

    tokio::spawn(async move { app.run().await.unwrap() });

    Ok(test_app)
}
