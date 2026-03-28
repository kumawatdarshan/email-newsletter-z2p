use crate::idempotency::IdempotencyKey;
use axum::{
    body::{Body, to_bytes},
    response::Response,
};
use repository::{
    Repository,
    idempotency::{HeaderPair, IdempotencyRepository},
};

pub async fn save_response(
    repo: &Repository,
    idempotency_key: &IdempotencyKey,
    user_id: &str,
    response: Response,
) -> anyhow::Result<Response> {
    let (parts, body) = response.into_parts();

    let status_code = parts.status;
    let bytes = to_bytes(body, usize::MAX).await?;

    let headers = {
        let mut h = Vec::with_capacity(parts.headers.len());
        for (key, value) in parts.headers.iter() {
            h.push(HeaderPair {
                key: key.as_str().to_owned(),
                value: value.as_bytes().to_owned(),
            });
        }
        h
    };

    repo.save_response(
        user_id,
        idempotency_key,
        status_code.as_u16(),
        headers,
        &bytes,
    )
    .await?;

    Ok(Response::from_parts(parts, Body::from(bytes)))
}
