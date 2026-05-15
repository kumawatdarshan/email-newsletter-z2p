use crate::{Connection, Repo, Result};

pub trait NewsletterRepository {
    fn get_confirmed_subscribers_raw(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<String>>> + Send;
}

impl<C: Connection> NewsletterRepository for Repo<C> {
    #[tracing::instrument(name = "Get Confirmed Subscribers", skip(self))]
    async fn get_confirmed_subscribers_raw(&self) -> Result<Vec<String>> {
        let mut conn = self.0.acquire().await?;
        let x = sqlx::query_scalar!(
            r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
        )
        .fetch_all(&mut *conn)
        .await?;

        Ok(x)
    }
}
