use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use zero2prod::config::read_config;
use zero2prod::startup::create_server;
use zero2prod::telemetry::configure_tracing;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // configure telemetry
    configure_tracing();

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
