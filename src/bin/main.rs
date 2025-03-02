use std::io::Error;
use zero2prod::config::read_config;
use zero2prod::startup::Application;
use zero2prod::telemetry::configure_tracing;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // configure telemetry
    let level = "info".to_string();
    let name = "zero2prod".to_string();
    let sink = std::io::stdout;
    configure_tracing(name, level, sink);
    tracing::info!("Starting zero2prod app ...");

    // read app config
    let config = read_config().expect("failed to read config");

    // spawn app
    let app = Application::launch(&config).await?;
    app.run_until_stopped().await?;
    Ok(())
}
