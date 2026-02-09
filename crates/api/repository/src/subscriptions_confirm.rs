use sqlx::SqlitePool;

pub struct SubscriptionsConfirmRepository;

impl SubscriptionsConfirmRepository {
    #[tracing::instrument(name = "Mark subscriber as Confirmed", skip(subscriber_id, pool))]
    pub async fn confirm_subscriber(
        pool: &SqlitePool,
        subscriber_id: String,
    ) -> Result<bool, sqlx::Error> {
        // TODO: ADD TIMESTAMP in schema TO AUTOMATICALLY invalidate token after 24h
        let result = sqlx::query!(
            r#"UPDATE subscriptions SET status = 'confirmed'
           WHERE id = $1 AND status = 'pending_confirmation'"#,
            subscriber_id
        )
        .execute(pool)
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

    #[tracing::instrument(name = "Get subscriber_id from token", skip(pool, subscription_token))]
    pub async fn get_subscriber_id_from_token(
        pool: &SqlitePool,
        subscription_token: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query!(
            r#"SELECT subscriber_id FROM subscription_tokens
           WHERE subscription_token = $1"#,
            subscription_token
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|r| r.subscriber_id))
    }
}
