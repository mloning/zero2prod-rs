use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use zero2prod::config::read_config;
use zero2prod::startup::create_server;
use zero2prod::telemetry::configure_tracing;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // configure telemetry
    let level = "info".to_string();
    let name = "zero2prod".to_string();
    let sink = std::io::stdout;
    configure_tracing(name, level, sink);

    // read app config
    let config = read_config().expect("failed to read config");

    // connect to database
    let connection_string = config.database.connection_string();
    let db_pool = PgPool::connect(connection_string.expose_secret())
        .await
        .expect("failed to connect to database");

    // bind to random port
    let address = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(address).expect("failed to bind random port");

    // launch server
    create_server(listener, db_pool)?.await
}
