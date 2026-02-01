use axum::Form;
use hmac::Mac;
use sqlx::SqlitePool;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode, header},
    response::IntoResponse,
};
use newsletter_macros::{DebugChain, IntoErrorResponse};
use secrecy::{ExposeSecret, SecretString};
use state::HmacSecret;

use crate::authentication::{AuthError, Credentials, validate_credentials};

#[derive(thiserror::Error, IntoErrorResponse, DebugChain)]
pub enum LoginError {
    #[error("Authentication failed.")]
    #[status(StatusCode::SEE_OTHER)]
    #[headers([header::LOCATION, "/login"])]
    AuthError(#[source] crate::authentication::AuthError),

    #[error("Something went wrong.")]
    #[status(StatusCode::SEE_OTHER)]
    #[headers([header::LOCATION, self.gen_redirect_with_error(hmac_secret)])]
    UnexpectedError {
        source: anyhow::Error,
        hmac_secret: HmacSecret,
    },
}

impl LoginError {
    fn gen_redirect_with_error(&self, secret: &HmacSecret) -> String {
        let query_string = format!("error={}", urlencoding::encode(&self.to_string()));

        type HmacSha256 = hmac::Hmac<sha2::Sha256>;
        let hmac_tag = {
            let mut mac = HmacSha256::new_from_slice(secret.expose_secret().as_bytes()).unwrap();

            mac.update(query_string.as_bytes());
            mac.finalize().into_bytes()
        };

        format!("/login?{query_string}&tag={hmac_tag:x}")
    }
}

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: SecretString,
}

#[tracing::instrument(
    skip(form, db_pool, hmac_secret),
    fields(username = tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    State(db_pool): State<SqlitePool>,
    State(hmac_secret): State<HmacSecret>,
    Form(form): Form<FormData>,
) -> Result<impl IntoResponse, LoginError> {
    let credentials = Credentials {
        username: form.username,
        password: form.password,
    };

    tracing::Span::current().record("username", tracing::field::display(&credentials.username));
    let user_id = validate_credentials(credentials, &db_pool)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials(_) => LoginError::AuthError(e),
            AuthError::UnexpectedError(_) => LoginError::UnexpectedError {
                source: e.into(),
                hmac_secret,
            },
        })?;
    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/")
        .body(Body::empty())
        .unwrap())
}
