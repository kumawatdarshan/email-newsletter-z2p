use crate::templates::JinjaEnv;
use crate::utils::auth_extractors::RequireAuth;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use minijinja::context;

pub async fn admin_dashboard(
    RequireAuth(user): RequireAuth,
    State(jinja): State<JinjaEnv>,
) -> Result<Response, Response> {
    let username = user.username;

    let template = jinja.get_template("admin_dashboard").unwrap();
    let html = template.render(context! { username }).unwrap();

    Ok((StatusCode::OK, Html(html)).into_response())
}
