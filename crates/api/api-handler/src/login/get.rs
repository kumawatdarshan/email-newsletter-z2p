use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use minijinja::context;

use crate::templates::JinjaEnv;

pub async fn login_form(flash: Messages, State(jinja): State<JinjaEnv>) -> impl IntoResponse {
    let messages: Vec<_> = flash.into_iter().map(|x| x.message).collect();
    let template = jinja.get_template("login").unwrap();
    let html = template.render(context! { messages }).unwrap();

    Html(html)
}
