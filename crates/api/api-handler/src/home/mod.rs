use crate::routes::routes_path::Index;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub async fn home(_: Index) -> impl IntoResponse {
    (StatusCode::OK, Html(include_str!("./home.html")))
}
