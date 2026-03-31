use crate::{Connection, Repo};

pub trait AdminDashboardRepository {
    fn get_username(
        &self,
        user_id: String,
    ) -> impl std::future::Future<Output = crate::Result<String>> + Send;
}

impl<C: Connection> AdminDashboardRepository for Repo<C> {
    #[tracing::instrument(name = "Get username", skip(self))]
    async fn get_username(&self, user_id: String) -> crate::Result<String> {
        let mut conn = self.0.acquire().await?;
        let result = sqlx::query!(
            r#"
              SELECT username  
              FROM users
              WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(result.username)
    }
}
