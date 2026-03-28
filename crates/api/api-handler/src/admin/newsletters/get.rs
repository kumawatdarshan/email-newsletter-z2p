use crate::templates::JinjaEnv;
use crate::utils::auth_extractors::{Authenticated, Browser};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum_messages::Messages;
use minijinja::context;

pub async fn newsletter_issue_form(
    flash: Messages,
    user: Authenticated<Browser>,
    State(jinja): State<JinjaEnv>,
) -> Result<Response, Response> {
    let username = &user.username;

    let messages: Vec<_> = flash.into_iter().map(|x| x.message).collect();
    let template = jinja.get_template("admin_newsletters").unwrap();
    let html = template.render(context! { messages, username }).unwrap();

    println!("{}", html);

    Ok((StatusCode::OK, Html(html)).into_response())
}
