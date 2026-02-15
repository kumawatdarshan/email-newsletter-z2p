use crate::Repository;

pub trait AdminDashboardRepository {
    fn get_username(
        &self,
        user_id: String,
    ) -> impl std::future::Future<Output = crate::Result<String>> + Send;
}

impl AdminDashboardRepository for Repository {
    #[tracing::instrument(name = "Get username", skip(self))]
    async fn get_username(&self, user_id: String) -> crate::Result<String> {
        let result = sqlx::query!(
            r#"
              SELECT username  
              FROM users
              WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(&self.0)
        .await?;

        Ok(result.username)
    }
}
