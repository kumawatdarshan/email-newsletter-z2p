use crate::routes::routes_path::Newsletters;
use anyhow::{Context, anyhow};
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use domain::{ConfirmedSubscriber, SubscriberEmail};
use email_client::EmailClient;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use repository::Repository;
use repository::newsletters::NewsletterRepository;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(Deserialize)]
struct Content {
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
    skip(body, repo, email_client),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub(crate) async fn publish_newsletter(
    _: Newsletters,
    State(repo): State<Repository>,
    State(email_client): State<EmailClient>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&repo).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(s) => {
                let email = &s.email;
                email_client
                    .send_email(email, &body.title, &body.content.html, &body.content.text)
                    .await
                    .with_context(|| format!("Failed to send newsletter to: {email}"))?;
            }
            // We did that convoluted Result<Vec<Result<X>,E>, E> so we could do this
            // By this I mean, separate warning logs for each invalid subscriber
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscriber, Using an invalid mail.",
                )
            }
        }
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
