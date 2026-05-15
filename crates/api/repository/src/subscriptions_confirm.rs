use crate::{Connection, Repo};

pub trait SubscriptionsConfirmRepository {
    fn confirm_subscriber(
        &self,
        subscriber_id: String,
    ) -> impl std::future::Future<Output = crate::Result<bool>> + Send;
    fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> impl std::future::Future<Output = crate::Result<Option<String>>> + Send;
}

impl<C: Connection> SubscriptionsConfirmRepository for Repo<C> {
    #[tracing::instrument(name = "Mark subscriber as Confirmed", skip(subscriber_id, self))]
    async fn confirm_subscriber(&self, subscriber_id: String) -> crate::Result<bool> {
        // TODO: ADD TIMESTAMP in schema TO AUTOMATICALLY invalidate token after 24h
        let mut conn = self.0.acquire().await?;
        let result = sqlx::query!(
            r#"UPDATE subscriptions SET status = 'confirmed'
           WHERE id = $1 AND status = 'pending_confirmation'"#,
            subscriber_id
        )
        .execute(&mut *conn)
        .await?;

        let was_updated = result.rows_affected() > 0;

        let message = if was_updated {
            "Successfully confirmed new subscriber"
        } else {
            "Subscriber already confirmed (idempotent operation)"
        };

        tracing::info!(
            %subscriber_id,
            message
        );

        Ok(was_updated)
    }

    #[tracing::instrument(name = "Get subscriber_id from token", skip(self, subscription_token))]
    async fn get_subscriber_id_from_token(
        &self,
        subscription_token: &str,
    ) -> crate::Result<Option<String>> {
        let mut conn = self.0.acquire().await?;
        let result = sqlx::query!(
            r#"SELECT subscriber_id FROM subscription_tokens
           WHERE subscription_token = $1"#,
            subscription_token
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(result.map(|r| r.subscriber_id))
    }
}
