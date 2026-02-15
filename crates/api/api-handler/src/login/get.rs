use axum::response::{Html, IntoResponse};
use axum_messages::Messages;
use std::fmt::Write;

pub async fn login_form(messages: Messages) -> impl IntoResponse {
    fn login_html(error_html: String) -> String {
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
            error_html
        )
    }

    let mut error_html = String::new();
    messages
        .into_iter()
        .for_each(|msg| writeln!(error_html, "<p><i>{}</i></p>", msg).unwrap());

    Html(login_html(error_html))
}
