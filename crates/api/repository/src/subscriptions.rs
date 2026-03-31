use crate::{Connection, Repo, Result};
use sqlx::types::{Uuid, chrono::Utc};
use types::NewSubscriber;

pub trait SubscriptionsRepository {
    fn insert_subscriber(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> impl std::future::Future<Output = Result<String>> + Send;
    fn store_token(
        &self,
        subscriber_id: &str,
        subscription_token: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;
}

impl<C: Connection> SubscriptionsRepository for Repo<C> {
    #[tracing::instrument(
        name = "Saving new subscriber details in the database.",
        skip(self, new_subscriber)
    )]
    async fn insert_subscriber(&self, new_subscriber: &NewSubscriber) -> Result<String> {
        let subscriber_id = Uuid::new_v4().to_string();
        let email = new_subscriber.email.as_ref();
        let name = new_subscriber.name.as_ref();
        let timestamp = Utc::now().to_string();

        let mut conn = self.0.acquire().await?;
        sqlx::query!(
            r#"
            INSERT INTO subscriptions (id, email,name, subscribed_at, status)
            VALUES ($1,$2,$3,$4, 'pending_confirmation')
            "#,
            subscriber_id,
            email,
            name,
            timestamp
        )
        .execute(&mut *conn)
        .await?;

        Ok(subscriber_id)
    }

    #[tracing::instrument(
        name = "Store subscription token in the database",
        skip(self, subscription_token)
    )]
    async fn store_token(&self, subscriber_id: &str, subscription_token: &str) -> Result<()> {
        let mut conn = self.0.acquire().await?;
        sqlx::query!(
            r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
            "#,
            subscription_token,
            subscriber_id,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}
