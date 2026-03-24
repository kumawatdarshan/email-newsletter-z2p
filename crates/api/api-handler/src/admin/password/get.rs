use crate::AppState;
use crate::auth_extractors::{Authenticated, Browser};
use crate::templates::JinjaEnv;
use axum::debug_handler;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum_messages::Messages;
use minijinja::context;
use repository::Repository;

#[debug_handler(state = AppState)]
pub async fn password_change_form(
    _: State<Repository>,
    flash: Messages,
    user: Authenticated<Browser>,
    State(jinja): State<JinjaEnv>,
) -> impl IntoResponse {
    let username = &user.username;

    let messages: Vec<_> = flash.into_iter().map(|x| x.message).collect();
    let template = jinja.get_template("change_password").unwrap();
    let html = template.render(context! { messages, username }).unwrap();

    (StatusCode::OK, Html(html))
}
