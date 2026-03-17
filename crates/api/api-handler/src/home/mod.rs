use crate::templates::JinjaEnv;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub async fn home(State(jinja): State<JinjaEnv>) -> impl IntoResponse {
    let template = jinja.get_template("home").unwrap();
    let html = template.render(minijinja::context! {}).unwrap();

    (StatusCode::OK, Html(html))
}
