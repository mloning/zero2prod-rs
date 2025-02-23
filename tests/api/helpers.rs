use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::config::{read_config, DatabaseConfig};
use zero2prod::email_client::EmailClient;
use zero2prod::startup::create_server;
use zero2prod::telemetry::configure_tracing;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn configure_database(config: &DatabaseConfig) -> PgPool {
    let connection_string = config.connection_string_without_db();
    let mut connection = PgConnection::connect_with(&connection_string)
        .await
        .expect("failed to connect to database");

    let query = format!(r#"CREATE DATABASE "{}";"#, config.name);
    connection
        .execute(query.as_str())
        .await
        .expect("failed to create database");

    let connection_string = config.connection_string();
    let db_pool = PgPool::connect_with(connection_string)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("failed to run migrations");

    db_pool
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let level = "info".to_string();
    let name = "zero2prod-test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        configure_tracing(name, level, std::io::stdout);
    } else {
        configure_tracing(name, level, std::io::sink);
    };
});

pub async fn spwan_app() -> TestApp {
    // configure tracing only once; all other calls are skipped
    Lazy::force(&TRACING);

    // read config
    let mut config = read_config().expect("failed to read config");

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

    // bind to random port
    let ip = "127.0.0.1";
    // setting port to 0 means the OS will assign a free port
    let listener = TcpListener::bind(format!("{}:0", ip)).expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port(); // free port assigned by OS
    let address = format!("http://{}:{}", ip, port);

    // set up database connection
    config.database.name = Uuid::new_v4().to_string(); // randomize database name for testing
    let db_pool = configure_database(&config.database).await;

    // create server
    let server =
        create_server(listener, db_pool.clone(), email_client).expect("failed to create server");

    // tokio::spawn spaws a new task (our server) when a new tokio runtime is launched and shuts
    // down all tasks when the runtime is stopped; tokio::test launches the new runtime
    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}
