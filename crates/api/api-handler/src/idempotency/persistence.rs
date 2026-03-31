use anyhow::Context;
use axum::body::Body;
use axum::body::to_bytes;
use axum::{http::StatusCode, response::Response};
use repository::{
    Repository,
    idempotency::{HeaderPair, IdempotencyRepository},
};
use types::IdempotencyKey;

pub async fn get_saved_response(
    repo: &Repository,
    idempotency_key: &IdempotencyKey,
    user_id: &str,
) -> anyhow::Result<Option<Response>> {
    let Some(r) = repo.get_saved_response(user_id, idempotency_key).await? else {
        return Ok(None);
    };

    tracing::error!("saved_response => {r:#?}");

    let status_code = StatusCode::from_u16(r.status_code.try_into()?)?;
    let mut builder = Response::builder().status(status_code);

    for HeaderPair { key, value } in r.response_headers {
        builder = builder.header(key, value);
    }

    let response = builder.body(r.response_body.into())?;
    Ok(Some(response))
}

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

pub enum NextAction {
    StartProcessing,
    ReturnSavedResponse(Response),
}

pub async fn try_processing(
    repo: &Repository,
    idempotency_key: &IdempotencyKey,
    user_id: &str,
) -> anyhow::Result<NextAction> {
    let txn = repo.as_ref().begin().await?;

    let n_inserted_rows = repo.num_of_inserted_rows(user_id, idempotency_key).await?;

    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing)
    } else {
        let saved_response = get_saved_response(repo, idempotency_key, user_id)
            .await?
            .with_context(|| "Expected a saved response, didn't find any")?;

        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}
