use anyhow::Context;
use axum::Router;
use configuration::Configuration;
use email_client::EmailClient;
use redis_client::create_redis_pool;
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

use crate::{AppState, routes::get_router};

pub struct Application {
    pub router: Router,
    pub listener: TcpListener,
}

impl Application {
    pub async fn build(config: &Configuration) -> anyhow::Result<Self> {
        ApplicationBuilder::new(config).build().await
    }

    pub fn address(&self) -> String {
        format!("http://{}", self.listener.local_addr().unwrap())
    }

    pub async fn run(self) -> anyhow::Result<()> {
        axum::serve(self.listener, self.router)
            .await
            .context("Failed to serve application using axum")
    }
}

pub struct ApplicationBuilder<'a> {
    config: &'a Configuration,
    db_pool: Option<SqlitePool>,
    email_client: Option<EmailClient>,
    redis_pool: Option<RedisPool>,
}

impl<'a> ApplicationBuilder<'a> {
    pub fn new(config: &'a Configuration) -> Self {
        Self {
            config,
            db_pool: None,
            email_client: None,
            redis_pool: None,
        }
    }

    pub fn with_db_pool(mut self, pool: SqlitePool) -> Self {
        self.db_pool = Some(pool);
        self
    }

    pub fn with_email_client(mut self, client: EmailClient) -> Self {
        self.email_client = Some(client);
        self
    }

    pub fn with_redis_pool(mut self, pool: RedisPool) -> Self {
        self.redis_pool = Some(pool);
        self
    }

    pub async fn build(self) -> anyhow::Result<Application> {
        let db_pool = match self.db_pool {
            Some(pool) => pool,
            None => SqlitePool::connect_lazy_with(self.config.database.options()),
        };

        let email_client = match self.email_client {
            Some(client) => client,
            None => EmailClient::from_config(&self.config.email_client),
        };

        let redis_pool = match self.redis_pool {
            Some(pool) => pool,
            None => create_redis_pool(&self.config.redis)
                .await
                .context("Failed to initialize redis pool")?,
        };

        let bind_addr = format!(
            "{}:{}",
            self.config.application.host, self.config.application.port
        );
        let listener = TcpListener::bind(&bind_addr)
            .await
            .context(format!("Failed to bind to address: {bind_addr}"))?;

        // local_addr returns only the domain.
        let base_url = format!("http://{}", listener.local_addr().unwrap());

        let app_state = AppState {
            db_pool,
            email_client,
            base_url,
        };

        let router = get_router(app_state, redis_pool)
            .await
            .expect("Failed to get router");

        Ok(Application { router, listener })
    }
}
