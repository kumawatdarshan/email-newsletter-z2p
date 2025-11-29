use anyhow::Context;
use axum::{Form, extract::State, http::StatusCode};
use domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use email_client::EmailClient;
use newsletter_macros::{DebugChain, IntoErrorResponse};
use rand::{Rng, distr::Alphanumeric};
use serde::Deserialize;
use sqlx::{
    Sqlite, Transaction,
    types::{Uuid, chrono::Utc},
};
use state::AppState;
use std::sync::Arc;

#[derive(Deserialize)]
pub(crate) struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;

        Ok(NewSubscriber { name, email })
    }
}

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum SubscribeError {
    #[error("{0}")]
    #[status(StatusCode::UNPROCESSABLE_ENTITY)]
    ValidationError(String),
    #[error(transparent)]
    #[status(StatusCode::INTERNAL_SERVER_ERROR)]
    UnexpectedError(#[from] anyhow::Error),
}

/// Handles new subscription requests.
///
/// # Responses
///
/// - **`201 Created`** — Subscription created successfully.
/// - **`422 Unprocessable Entity`** — Input validation failed.
/// - **`500 Internal Server Error`** — One of the following operations failed:
///   - Inserting the subscriber into the database.
///   - Storing the `subscription_token`.
///   - Sending the confirmation email.
#[tracing::instrument(
    name = "Adding a new Subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub(crate) async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> Result<StatusCode, SubscribeError> {
    let new_subscriber = form.try_into().map_err(SubscribeError::ValidationError)?;

    let mut transaction = state
        .db_pool
        .begin()
        .await
        .context("Failed to begin database transaction")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber in the database")?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for the new subscriber")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    send_confirmation_email(
        &state.email_client,
        new_subscriber,
        &state.base_url,
        &subscription_token,
    )
    .await
    .context("Faield to send a confirmation mail")?;

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database.",
    skip(transaction, new_subscriber)
)]
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Sqlite>,
    new_subscriber: &NewSubscriber,
) -> Result<String, sqlx::Error> {
    let subscriber_id = Uuid::new_v4().to_string();
    let email = new_subscriber.email.as_ref();
    let name = new_subscriber.name.as_ref();
    let timestamp = Utc::now().to_string();

    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email,name, subscribed_at, status)
            VALUES ($1,$2,$3,$4, 'pending_confirmation')
        "#,
        subscriber_id,
        email,
        name,
        timestamp
    )
    .execute(&mut **transaction)
    .await?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Sending Confirmation mail",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    // TODO: LOGS SHOULD EMIT IF I MISSED A ARG
    let confirmation_link =
        format!("{base_url}/subscribe/confirm?subscription_token={subscription_token}");

    let html = format!(
        "Welcome to our newsletter!<br />\
    Click <a href=\"{confirmation_link}\">here</a> to confirm your subscription."
    );

    let plaintext = format!(
        "Welcome to our newsletter!\
        Visit {confirmation_link} to confirm your subscription."
    );

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &html, &plaintext)
        .await
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
async fn store_token(
    transaction: &mut Transaction<'_, Sqlite>,
    subscriber_id: String,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscription_tokens (subscription_token, subscriber_id)
            VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    )
    .execute(&mut **transaction) // this is some black magic
    .await?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = rand::rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
