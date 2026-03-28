use super::IdempotencyKey;
use axum::{http::StatusCode, response::Response};
use repository::{
    Repository,
    idempotency::{HeaderPair, IdempotencyRepository},
};

pub async fn get_saved_response(
    repo: &Repository,
    idempotency_key: &IdempotencyKey,
    user_id: &str,
) -> anyhow::Result<Option<Response>> {
    let Some(r) = repo.get_saved_response(user_id, idempotency_key).await? else {
        return Ok(None);
    };

    let status_code = StatusCode::from_u16(r.status_code.try_into()?)?;
    let mut builder = Response::builder().status(status_code);

    for HeaderPair { key, value } in r.response_headers {
        builder = builder.header(key, value);
    }

    let response = builder.body(r.response_body.into())?;
    Ok(Some(response))
}
