use crate::{
    app_state::AppFactory,
    configuration::{AppState, DatabaseConfiguration},
    routes::get_router,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::Result;

/// Only for integration tests.
#[derive(Debug)]
pub struct TestAppState {
    pub addr: String,
    pub db_pool: PgPool,
}

/// Creating a uuid named db through `PgConnection` and then doing the migrations through `PgPool`
pub async fn configure_test_database(settings: &DatabaseConfiguration) -> PgPool {
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

pub async fn spawn_app_testing() -> Result<TestAppState> {
    let mut app_factory = AppFactory::new(true)?.init_subscriber()?;
    let listener = app_factory.create_listener().await?;
    let db_pool = app_factory.create_db_pool().await;
    let email_client = app_factory.create_email_client();

    let app_state = AppState {
        db_pool: db_pool.clone(),
        email_client,
    };

    let test_app = TestAppState {
        addr: format!("http://{}", listener.local_addr()?),
        db_pool,
    };

    let router = get_router(app_state);

    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Test server failed");
    });

    Ok(test_app)
}
