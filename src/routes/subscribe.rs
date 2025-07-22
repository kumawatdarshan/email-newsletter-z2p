use std::sync::Arc;

use axum::{Form, debug_handler, extract::State, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::configuration::AppState;

#[derive(Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[debug_handler]
pub async fn subscribe(
    State(state): State<Arc<AppState>>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    let result = sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email,name, subscribed_at)
            VALUES ($1,$2,$3,$4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&state.db_pool)
    .await;

    match result {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            eprintln!("Failed to execute query: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
