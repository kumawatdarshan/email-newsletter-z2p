use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use sqlx::types::{Uuid, chrono::Utc};
use std::sync::Arc;

use crate::{
    configuration::AppState,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
};

#[tracing::instrument(
    name = "Saving new subscriber details in the database."
    skip(state, form)
)]
async fn insert_subscriber(state: Arc<AppState>, form: &NewSubscriber) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email,name, subscribed_at, status)
            VALUES ($1,$2,$3,$4, 'confirmed')
        "#,
        Uuid::new_v4(),
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    )
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {e:#?}");
        e
    })?;

    Ok(())
}

#[derive(Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(form.name)?;
        let email = SubscriberEmail::parse(form.email)?;

        Ok(NewSubscriber { name, email })
    }
}

/// - `201 Created` on success
/// - `422 UNPROCESSABLE_ENTITY` if the input validation fails
/// - `500 Internal Server Error` if inserting into the database fails
#[tracing::instrument(
    name = "Adding a new Subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
#[axum::debug_handler]
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, StatusCode> {
    let new_subscriber = form
        .try_into()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    insert_subscriber(state, &new_subscriber)
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
