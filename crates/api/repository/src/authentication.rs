use crate::Repository;
use crate::Result;
use secrecy::SecretString;

pub trait AuthenticationRepository {
    fn get_stored_credentials(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<(String, SecretString)>>> + Send;
}

impl AuthenticationRepository for Repository {
    #[tracing::instrument(name = "Get Stored Credentials", skip(username, self))]
    async fn get_stored_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(String, SecretString)>> {
        let row = sqlx::query!(
            r#"
           SELECT user_id, password_hash
           from users
           WHERE username = $1
        "#,
            username,
        )
        .fetch_optional(&self.0)
        .await?
        .map(|row| (row.user_id, SecretString::new(row.password_hash.into())));

        Ok(row)
    }
}
