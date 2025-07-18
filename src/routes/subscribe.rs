use axum::{Form, http::StatusCode};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub async fn subscribe(Form(form_data): Form<FormData>) -> StatusCode {
    StatusCode::OK
}
