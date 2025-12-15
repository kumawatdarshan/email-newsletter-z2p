use axum::{
    Form,
    body::Body,
    http::{Response, StatusCode, header::LOCATION},
    response::IntoResponse,
};
use secrecy::SecretString;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

pub async fn login(Form(form): Form<FormData>) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(LOCATION, "/")
        .body(Body::empty())
        .unwrap()
}
