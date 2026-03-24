use crate::auth_extractors::{Authenticated, Browser};
use crate::templates::JinjaEnv;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use minijinja::context;

pub async fn admin_dashboard(
    user: Authenticated<Browser>,
    State(jinja): State<JinjaEnv>,
) -> Result<Response, Response> {
    let username = &user.username;

    let template = jinja.get_template("admin_dashboard").unwrap();
    let html = template.render(context! { username }).unwrap();

    Ok((StatusCode::OK, Html(html)).into_response())
}
