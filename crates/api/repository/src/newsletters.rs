use crate::Result;

use crate::Repository;

pub trait NewsletterRepository {
    fn get_confirmed_subscribers_raw(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<String>>> + Send;
}

impl NewsletterRepository for Repository {
    #[tracing::instrument(name = "Get Confirmed Subscribers", skip(self))]
    async fn get_confirmed_subscribers_raw(&self) -> Result<Vec<String>> {
        sqlx::query_scalar!(
            r#"
            SELECT email
            FROM subscriptions
            WHERE status = 'confirmed'
        "#
        )
        .fetch_all(&self.0)
        .await
    }
}
