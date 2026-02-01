use axum::extract::{RawQuery, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use state::HmacSecret;

#[derive(serde::Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String, anyhow::Error> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;

        Ok(self.error)
    }
}

pub async fn login_form(
    RawQuery(query_string): RawQuery,
    State(hmac_secret): State<HmacSecret>,
) -> impl IntoResponse {
    fn login_html(error_html: Option<String>) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
  <head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Login</title>
  </head>
  <body>
    {}
    <form method="post">
      <label>Username
        <input
          type="text"
          placeholder="Enter Username"
          name="username"
        >
      </label>
      <label>Password
        <input
          type="password"
          placeholder="Enter Password"
          name="password"
        >
      </label>

      <button type="submit">Login</button>
    </form>
  </body>
</html>"#,
            error_html.unwrap_or_default()
        )
    }

    let Some(qs) = query_string else {
        return (StatusCode::OK, Html(login_html(None)));
    };

    let Ok(query) = serde_urlencoded::from_str::<QueryParams>(&qs) else {
        tracing::warn!("Failed to deserialize query parameters");
        return (
            StatusCode::OK,
            Html(login_html(Some(
                "Failed to deserialize query parameters".into(),
            ))),
        );
    };

    let error_html = if let Ok(error) = query.verify(&hmac_secret) {
        format!("<p><i>{}</i></p>", htmlescape::encode_minimal(&error))
    } else {
        tracing::warn!("Failed to verify query parameters using HMAC tag");
        return (
            StatusCode::OK,
            Html(login_html(Some(
                "Failed to verify query parameters using HMAC tag".into(),
            ))),
        );
    };

    (StatusCode::OK, Html(login_html(Some(error_html))))
}
