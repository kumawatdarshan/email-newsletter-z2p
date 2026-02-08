use anyhow::{Context, Result};
use api::Application;
use configuration::get_configuration;

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init_tracing().context("Failed to initialize tracing.")?;
    let config = get_configuration().context("Failed to read Configuration.")?;

    let app = Application::build(&config).await?;
    app.run().await?;

    Ok(())
}
