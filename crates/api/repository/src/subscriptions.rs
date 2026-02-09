use domain::NewSubscriber;
use sqlx::{
    Sqlite, Transaction,
    types::{Uuid, chrono::Utc},
};

pub struct SubscriptionsRepository;

impl SubscriptionsRepository {
    #[tracing::instrument(
        name = "Saving new subscriber details in the database.",
        skip(transaction, new_subscriber)
    )]
    pub async fn insert_subscriber(
        transaction: &mut Transaction<'_, Sqlite>,
        new_subscriber: &NewSubscriber,
    ) -> Result<String, sqlx::Error> {
        let subscriber_id = Uuid::new_v4().to_string();
        let email = new_subscriber.email.as_ref();
        let name = new_subscriber.name.as_ref();
        let timestamp = Utc::now().to_string();

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
        .execute(&mut **transaction)
        .await?;

        Ok(subscriber_id)
    }

    #[tracing::instrument(
        name = "Store subscription token in the database",
        skip(transaction, subscription_token)
    )]
    pub async fn store_token(
        transaction: &mut Transaction<'_, Sqlite>,
        subscriber_id: &str,
        subscription_token: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
            "#,
            subscription_token,
            subscriber_id,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }
}
