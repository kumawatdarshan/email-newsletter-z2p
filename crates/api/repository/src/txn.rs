use crate::{Connection, Repo, Repository, Result};
use futures_util::future::BoxFuture;
use sqlx::{Sqlite, SqliteConnection};
use std::ops::{Deref, DerefMut};
use tokio::sync::{Mutex, MutexGuard};

type TxnMutex = Mutex<OwnedTxn>;

impl Connection for TxnMutex {
    type Guard<'a> = TxnGuard<'a>;

    fn acquire(&self) -> BoxFuture<'_, Result<TxnGuard<'_>>> {
        Box::pin(async { Ok(TxnGuard(self.lock().await)) })
    }
}

pub struct OwnedTxn(sqlx::Transaction<'static, Sqlite>);

impl Deref for OwnedTxn {
    type Target = SqliteConnection;
    fn deref(&self) -> &SqliteConnection {
        &self.0
    }
}

impl DerefMut for OwnedTxn {
    fn deref_mut(&mut self) -> &mut SqliteConnection {
        &mut self.0
    }
}

pub struct TxnGuard<'a>(MutexGuard<'a, OwnedTxn>);

impl Deref for TxnGuard<'_> {
    type Target = SqliteConnection;
    fn deref(&self) -> &SqliteConnection {
        &self.0
    }
}

impl DerefMut for TxnGuard<'_> {
    fn deref_mut(&mut self) -> &mut SqliteConnection {
        &mut self.0
    }
}

pub type TransactionalRepository = Repo<TxnMutex>;

impl Repository {
    /// Typestate switch
    pub async fn begin(&self) -> Result<TransactionalRepository> {
        let tx = self.0.begin().await?;
        Ok(Repo(Mutex::new(OwnedTxn(tx))))
    }
}

impl TransactionalRepository {
    pub async fn commit(self) -> Result<()> {
        self.0.into_inner().0.commit().await?;
        Ok(())
    }
}
