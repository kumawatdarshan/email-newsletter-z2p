use anyhow::Context;
use axum::body::Body;
use axum::body::to_bytes;
use axum::{http::StatusCode, response::Response};
use repository::{
    Repository, TransactionalRepository,
    idempotency::{HeaderPair, IdempotencyRepository},
};
use types::IdempotencyKey;

#[tracing::instrument(name = "Fetching saved response(idempotency)", skip(repo))]
pub async fn get_saved_response(
    repo: &impl IdempotencyRepository,
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

#[tracing::instrument(name = "Saving response(idempotency)", skip(txn, response))]
pub async fn save_response(
    txn: TransactionalRepository,
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

    txn.save_response(
        user_id,
        idempotency_key,
        status_code.as_u16(),
        headers,
        &bytes,
    )
    .await?;

    txn.commit()
        .await
        .context("Failed to commit idempotency transaction")?;

    Ok(Response::from_parts(parts, Body::from(bytes)))
}

pub enum NextAction {
    StartProcessing(TransactionalRepository),
    ReturnSavedResponse(Response),
}

pub async fn try_processing(
    repo: &Repository,
    idempotency_key: &IdempotencyKey,
    user_id: &str,
) -> anyhow::Result<NextAction> {
    let txn = repo.begin().await?;

    let n_inserted_rows = txn.num_of_inserted_rows(user_id, idempotency_key).await?;

    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing(txn))
    } else {
        let saved_response = get_saved_response(&txn, idempotency_key, user_id)
            .await?
            .with_context(|| "Expected a saved response, didn't find any")?;

        Ok(NextAction::ReturnSavedResponse(saved_response))
    }
}
