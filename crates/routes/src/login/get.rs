use axum::response::{Html, IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn login_form(jar: CookieJar) -> impl IntoResponse {
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

    let error_html = jar
        .get("_flash")
        .map(|x| format!("<p><i>{}</i></p>", x.value()));

    // not using the hack of max age duration zero
    // when there is first class support like this.
    let jar = jar.remove("_flash");

    (jar, Html(login_html(error_html)))
}
