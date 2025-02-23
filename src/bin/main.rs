use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use zero2prod::config::read_config;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::create_server;
use zero2prod::telemetry::configure_tracing;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // configure telemetry
    let level = "info".to_string();
    let name = "zero2prod".to_string();
    let sink = std::io::stdout;
    configure_tracing(name, level, sink);
    tracing::info!("Starting zero2prod ...");

    // read app config
    tracing::info!("Reading config ...");
    let config = read_config().expect("failed to read config");

    // set up email client
    let timeout = config.email_client.parse_timeout();
    let sender_email = config
        .email_client
        .parse_sender_email()
        .expect("could not parse sender email");
    let email_client = EmailClient::new(
        config.email_client.base_url,
        sender_email,
        timeout,
        config.email_client.auth_token,
    );

    // set up database connection, with lazy connection when used for the first time
    tracing::info!("Setting up database connection ...");
    let connection_string = config.database.connection_string();
    let db_pool = PgPool::connect_lazy_with(connection_string);

    // bind to random port
    let address = format!("{}:{}", config.app.host, config.app.port);
    let listener = TcpListener::bind(address).expect("failed to bind random port");

    // launch server
    tracing::info!("Launching server ...");
    create_server(listener, db_pool, email_client)?.await
}
