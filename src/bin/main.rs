use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{filter::EnvFilter, layer::SubscriberExt, Registry};
use zero2prod::config::read_config;
use zero2prod::startup::create_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // configure logging
    // redirect all log's events to our tracing subscriber
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");

    // read app config
    let config = read_config().expect("failed to read config");

    // connect to database
    let connection_string = config.database.connection_string();
    let db_pool = PgPool::connect(&connection_string)
        .await
        .expect("failed to connect to database");

    // bind to random port
    let address = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(address).expect("failed to bind random port");

    // launch server
    create_server(listener, db_pool)?.await
}
