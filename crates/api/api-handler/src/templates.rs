pub type JinjaEnv = minijinja::Environment<'static>;

pub fn get_jinja_env() -> JinjaEnv {
    let mut env = JinjaEnv::new();
    env.set_auto_escape_callback(|name| {
        if name.ends_with(".html") {
            minijinja::AutoEscape::Html
        } else {
            minijinja::AutoEscape::None
        }
    });

    env.add_template("home", include_str!("templates/home.jinja.html"))
        .unwrap();
    env.add_template("login", include_str!("templates/login.jinja.html"))
        .unwrap();
    env.add_template(
        "admin_dashboard",
        include_str!("templates/admin_dashboard.jinja.html"),
    )
    .unwrap();
    env.add_template(
        "change_password",
        include_str!("templates/change_password.jinja.html"),
    )
    .unwrap();

    env
}
