use axum::{
    Form,
    extract::State,
    response::{IntoResponse, Redirect},
};
use axum_messages::Messages;
use repository::Repository;
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::{AuthenticatedUser, routes_path::ADMIN_PASSWORD};

#[derive(Deserialize)]
pub struct FormData {
    current_password: SecretString,
    new_password: SecretString,
    new_password_check: SecretString,
}

/// This returns 401 instead of 303 because this is a post request.
/// It is not a `frontend` facing route
pub async fn change_password(
    _: AuthenticatedUser,
    flash: Messages,
    State(_repo): State<Repository>,
    Form(form): Form<FormData>,
) -> impl IntoResponse {
    if form.new_password.expose_secret() != form.new_password_check.expose_secret() {
        flash.error("You entered two different new passwords - the field values must match.");
        return Redirect::to(ADMIN_PASSWORD).into_response();
    }

    "TODO".into_response()
}
