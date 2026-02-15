pub mod admin_dashboard;
pub mod authentication;
pub mod newsletters;
pub mod signup;
pub mod subscriptions;
pub mod subscriptions_confirm;

pub(crate) type Result<T> = core::result::Result<T, sqlx::Error>;

#[derive(Debug, Clone)]
pub struct Repository(sqlx::SqlitePool);

impl AsRef<sqlx::SqlitePool> for Repository {
    fn as_ref(&self) -> &sqlx::SqlitePool {
        &self.0
    }
}

impl AsMut<sqlx::SqlitePool> for Repository {
    fn as_mut(&mut self) -> &mut sqlx::SqlitePool {
        &mut self.0
    }
}

// TODO: ts is ugly, its so wrong ik it.
// But idk how to solve it
// Problem being, the wrapper hiding away functions
impl Repository {
    pub fn new(pool: sqlx::SqlitePool) -> Self {
        Self(pool)
    }

    pub fn connect_lazy_with(options: sqlx::sqlite::SqliteConnectOptions) -> Self {
        Self(sqlx::SqlitePool::connect_lazy_with(options))
    }

    // You might also want:
    pub async fn connect_with(options: sqlx::sqlite::SqliteConnectOptions) -> Result<Self> {
        Ok(Self(sqlx::SqlitePool::connect_with(options).await?))
    }
}
