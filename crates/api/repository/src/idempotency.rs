use crate::Repository;
use crate::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct HeaderPair {
    pub key: String,
    pub value: Vec<u8>,
}

#[derive(Debug)]
pub struct SavedResponse {
    pub status_code: i64,
    pub response_body: Vec<u8>,
    pub response_headers: Vec<HeaderPair>,
}

pub trait IdempotencyRepository {
    fn get_saved_response(
        &self,
        user_id: &str,
        idempotency_key: &str,
    ) -> impl std::future::Future<Output = Result<Option<SavedResponse>>> + Send;

    fn save_response(
        &self,
        user_id: &str,
        idempotency_key: &str,
        status_code: u16,
        headers: Vec<HeaderPair>,
        body: &[u8],
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    fn n_inserted_rows(
        &self,
        user_id: &str,
        idempotency_key: &str,
    ) -> impl std::future::Future<Output = Result<u64>> + Send;
}

impl IdempotencyRepository for Repository {
    #[tracing::instrument(name = "Get Saved Idempotency Response", skip(self))]
    async fn get_saved_response(
        &self,
        user_id: &str,
        idempotency_key: &str,
    ) -> Result<Option<SavedResponse>> {
        let Some(row) = sqlx::query!(
            r#"
                -- ! syntax is the not null assertion
                SELECT
                    response_status_code as "response_status_code!", 
                    response_body as "response_body!",
                    response_headers as "response_headers!"
                FROM idempotency
                WHERE user_id = $1 AND idempotency_key = $2
            "#,
            user_id,
            idempotency_key
        )
        .fetch_optional(&self.0)
        .await?
        else {
            return Ok(None);
        };

        Ok(Some(SavedResponse {
            status_code: row.response_status_code,
            response_body: row.response_body,
            response_headers: serde_json::from_str(&row.response_headers).unwrap_or_default(),
        }))
    }

    #[tracing::instrument(name = "Saved Idempotency Response", skip(self))]
    async fn save_response(
        &self,
        user_id: &str,
        idempotency_key: &str,
        status_code: u16,
        headers: Vec<HeaderPair>,
        body: &[u8],
    ) -> Result<()> {
        let headers_json = serde_json::to_string(&headers)?;
        tracing::error!("save_headers => {:#?}", headers_json);

        sqlx::query!(
            r#"
                UPDATE idempotency 
                SET
                    response_status_code = $3,
                    response_body = $4,
                    response_headers = $5
                 WHERE
                    user_id = $1 AND
                    idempotency_key = $2
            "#,
            user_id,
            idempotency_key,
            status_code,
            body,
            headers_json
        )
        .execute(&self.0)
        .await?;

        Ok(())
    }

    async fn n_inserted_rows(&self, user_id: &str, idempotency_key: &str) -> Result<u64> {
        let res = sqlx::query!(
            r#"
                INSERT INTO idempotency (
                    user_id,
                    idempotency_key,
                    created_at 
                )
                VALUES ($1, $2, datetime('now'))
                ON CONFLICT
                DO NOTHING
            "#,
            user_id,
            idempotency_key
        )
        .execute(&self.0)
        .await?
        .rows_affected();

        Ok(res)
    }
}
