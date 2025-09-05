use z2p::{app_state::AppBuilder, routes::get_router};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut app_builder = AppBuilder::new(false)?.init_subscriber()?;
    let (listener, app_state) = app_builder.build_app_state().await?;

    let router = get_router(app_state);

    axum::serve(listener, router).await
}
