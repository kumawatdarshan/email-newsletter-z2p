use crate::{
    app_state::AppBuilder,
    configuration::{AppState, DatabaseConfiguration},
    routes::get_router,
};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::io::Result;

/// Only for integration tests.
#[derive(Debug)]
pub struct TestApp {
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

pub async fn spawn_app_testing() -> Result<TestApp> {
    let mut app_builder = AppBuilder::new(true)?.init_subscriber()?;
    let listener = app_builder.create_listener().await?;
    let address = listener.local_addr()?;
    let pool = app_builder.create_db_pool().await;
    let email_client = app_builder.create_email_client();

    let test_app = TestApp {
        addr: format!("http://{address}"),
        db_pool: pool.clone(),
    };

    let app_state = AppState {
        db_pool: pool,
        email_client,
    };

    let router = get_router(app_state);

    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("Test server failed");
    });

    Ok(test_app)
}
