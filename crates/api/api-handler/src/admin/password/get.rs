use crate::AppState;
use crate::templates::JinjaEnv;
use crate::utils::auth_extractors::RequireAuth;
use axum::debug_handler;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum_messages::Messages;
use minijinja::context;
use repository::Repository;

#[debug_handler(state = AppState)]
pub async fn password_change_form(
    _: RequireAuth,
    _: State<Repository>,
    flash: Messages,
    State(jinja): State<JinjaEnv>,
) -> impl IntoResponse {
    let messages: Vec<_> = flash.into_iter().map(|x| x.message).collect();
    let template = jinja.get_template("change_password").unwrap();
    let html = template.render(context! { messages }).unwrap();

    (StatusCode::OK, Html(html))
}
