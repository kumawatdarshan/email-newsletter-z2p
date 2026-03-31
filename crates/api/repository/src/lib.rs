pub mod admin_dashboard;
pub mod authentication;
pub mod idempotency;
pub mod newsletters;
pub mod signup;
pub mod subscriptions;
pub mod subscriptions_confirm;
mod txn;

use futures_util::future::BoxFuture;
use sqlx::{Sqlite, SqliteConnection, pool::PoolConnection};
use std::ops::DerefMut;

pub use txn::TransactionalRepository;

#[derive(thiserror::Error, Debug)]
pub enum RepoError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("An unexpected error occurred: {0}")]
    UnexpectedError(String),
}

pub(crate) type Result<T> = core::result::Result<T, RepoError>;

/// Provides access to a `SqliteConnection`.
///
/// Implemented for `SqlitePool` (auto-commit) and
/// `Mutex<Transaction<'static, Sqlite>>` (transactional).
pub trait Connection: Send + Sync {
    /// A guard that derefs to `SqliteConnection`.
    type Guard<'a>: DerefMut<Target = SqliteConnection> + Send + 'a
    where
        Self: 'a;

    /// Acquire a connection handle.
    fn acquire(&self) -> BoxFuture<'_, Result<Self::Guard<'_>>>;
}

impl Connection for sqlx::SqlitePool {
    type Guard<'a> = PoolConnection<Sqlite>;

    fn acquire(&self) -> BoxFuture<'_, Result<PoolConnection<Sqlite>>> {
        Box::pin(async { Ok(sqlx::SqlitePool::acquire(self).await?) })
    }
}

pub struct Repo<C: Connection>(C);
pub type Repository = Repo<sqlx::SqlitePool>;

impl Repository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self(pool)
    }

    pub fn connect_lazy_with(options: sqlx::sqlite::SqliteConnectOptions) -> Self {
        Self(sqlx::SqlitePool::connect_lazy_with(options))
    }

    pub async fn connect_with(options: sqlx::sqlite::SqliteConnectOptions) -> Result<Self> {
        Ok(Self(sqlx::SqlitePool::connect_with(options).await?))
    }
}

impl Clone for Repository {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl std::fmt::Debug for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Repository").field(&self.0).finish()
    }
}

impl AsRef<sqlx::SqlitePool> for Repository {
    fn as_ref(&self) -> &sqlx::SqlitePool {
        &self.0
    }
}
