use crate::{
    auth_extractors::{Api, Authenticated},
    idempotency::{NextAction, save_response, try_processing},
    routes_path::ADMIN_NEWSLETTERS,
};
use anyhow::{Context, anyhow};
use axum::response::IntoResponse;
use axum::{Form, extract::State, http::StatusCode, response::Redirect};
use axum_messages::Messages;
use email_client::EmailClient;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::{Repository, newsletters::NewsletterRepository};
use serde::Deserialize;
use types::{ConfirmedSubscriber, IdempotencyKey, SubscriberEmail};

#[derive(Deserialize)]
pub struct FormData {
    title: String,
    html: String,
    text: String,
    idempotency_key: String,
}

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum PublishError {
    #[error("Bad Request")]
    #[status(StatusCode::BAD_REQUEST)]
    BadRequest,
    #[error("Something went wrong.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, repo, email_client, user),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub(crate) async fn publish_newsletter(
    user: Authenticated<Api>,
    flash: Messages,
    State(repo): State<Repository>,
    State(email_client): State<EmailClient>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, PublishError> {
    fn success_flash(flash: Messages) {
        flash.info("The newsletter issue has been published!");
    }

    let idempotency_key: IdempotencyKey = form
        .idempotency_key
        .try_into()
        .map_err(|_| PublishError::BadRequest)?;

    match try_processing(&repo, &idempotency_key, &user.user_id).await? {
        NextAction::StartProcessing => {}
        NextAction::ReturnSavedResponse(saved_response) => {
            success_flash(flash);
            return Ok(saved_response);
        }
    }

    let subscribers = get_confirmed_subscribers(&repo).await?;

    for subscriber in subscribers {
        let Ok(subscriber) = subscriber else {
            // We did that convoluted Result<Vec<Result<X>,E>, E> so we could do this
            // By this I mean, separate warning logs for each invalid subscriber
            tracing::warn!(
                error.cause_chain = ?subscriber.unwrap_err(),
                "Skipping a confirmed subscriber, Details are invalid.",
            );
            continue;
        };
        email_client
            .send_email(&subscriber.email, &form.title, &form.html, &form.text)
            .await
            .with_context(|| format!("Failed to send newsletter to: {}", subscriber.email))?;
    }

    success_flash(flash);
    let response = save_response(
        &repo,
        &idempotency_key,
        &user.user_id,
        // how to verify what error point returns what error? like this should resolve to 500 because it makes sql connection
        Redirect::to(ADMIN_NEWSLETTERS).into_response(),
    )
    .await?;

    Ok(response)
}

#[tracing::instrument(name = "Get Confirmed Subscribers", skip(repo))]
async fn get_confirmed_subscribers(
    repo: &Repository,
) -> anyhow::Result<Vec<anyhow::Result<ConfirmedSubscriber>>> {
    let rows = repo.get_confirmed_subscribers_raw().await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|x| {
            SubscriberEmail::parse(x)
                .map(|email| ConfirmedSubscriber { email })
                .map_err(|error| anyhow!(error))
        })
        .collect();

    Ok(confirmed_subscribers)
}
