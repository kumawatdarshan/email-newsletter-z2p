use crate::auth_extractors::{Api, Authenticated};
use anyhow::{Context, anyhow};
use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use domain::{ConfirmedSubscriber, SubscriberEmail};
use email_client::EmailClient;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::{Repository, newsletters::NewsletterRepository};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    title: String,
    html: String,
    text: String,
}

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum PublishError {
    #[error("Something went wrong.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(form, repo, email_client),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub(crate) async fn publish_newsletter(
    _: Authenticated<Api>,
    State(repo): State<Repository>,
    State(email_client): State<EmailClient>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&repo).await?;

    for subscriber in subscribers {
        let Ok(subscriber) = subscriber else {
            // We did that convoluted Result<Vec<Result<X>,E>, E> so we could do this
            // By this I mean, separate warning logs for each invalid subscriber
            tracing::warn!(
                error.cause_chain = ?subscriber.unwrap_err(),
                "Skipping a confirmed subscriber, Using an invalid mail.",
            );
            continue;
        };
        email_client
            .send_email(&subscriber.email, &form.title, &form.html, &form.text)
            .await
            .with_context(|| format!("Failed to send newsletter to: {}", subscriber.email))?;
    }

    Ok(StatusCode::OK)
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
