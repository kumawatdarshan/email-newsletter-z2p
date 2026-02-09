use crate::authentication::{AuthError, basic_authentication, validate_credentials};
use anyhow::{Context, anyhow};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
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
    // this will delegate error to the super::authentication::AuthError
    #[error("Authentication Failed.")]
    #[status(StatusCode::UNAUTHORIZED)]
    #[headers([header::WWW_AUTHENTICATE , r#"Basic realm="publish""#])]
    AuthError(#[source] crate::authentication::AuthError),

    #[error("Something went wrong.")]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, repo, email_client, headers),
    fields(username = tracing::field::Empty, user_id = tracing::field::Empty)
)]
pub(crate) async fn publish_newsletter(
    headers: HeaderMap,
    State(repo): State<Repository>,
    State(email_client): State<EmailClient>,
    Json(body): Json<BodyData>,
) -> Result<impl IntoResponse, PublishError> {
    let credentials = basic_authentication(headers).map_err(PublishError::AuthError)?;

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));

    let user_id = validate_credentials(&repo, credentials)
        .await
        // because it also makes sql queries which can fail outside the boundary of invalid credentials
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e),
            AuthError::UnexpectedError(source) => PublishError::UnexpectedError(source),
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

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
