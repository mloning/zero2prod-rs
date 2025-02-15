use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::config::{read_config, DatabaseConfig};
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

async fn spwan_app() -> TestApp {
    // configure tracing only once; all other calls are skipped
    Lazy::force(&TRACING);

    let ip = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", ip)).expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port(); // free port assigned by OS
    let address = format!("http://{}:{}", ip, port);

    let mut config = read_config().expect("failed to read config");
    config.database.name = Uuid::new_v4().to_string(); // randomize database name for testing

    let db_pool = configure_database(&config.database).await;
    let server = create_server(listener, db_pool.clone()).expect("failed to create server");

    // tokio::spawn spaws a new task (our server) when a new tokio runtime is launched and shuts
    // down all tasks when the runtime is stopped; tokio::test launches the new runtime
    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}

#[tokio::test]
async fn health_check_returns_200_and_no_body() {
    // arange, start app and create a client
    let app = spwan_app().await;
    let client = reqwest::Client::new();

    // act, send request
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(200, response.status().as_u16());
    assert_eq!(Some(0), response.content_length()); // no body
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    // arange, start app and create a client
    let app = spwan_app().await;
    let client = reqwest::Client::new();

    // act, send request
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(200, response.status().as_u16());

    let subscription = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch data from database");

    assert_eq!(subscription.name, "le guin");
    assert_eq!(subscription.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_data() {
    // arange, start app and create a client
    let app = spwan_app().await;
    let client = reqwest::Client::new();
    // TODO move test cases into parametrized fixture, using rtest crate
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // act, send request
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // assert, check response
        assert!(response.status().is_client_error()); // 200 status
        assert_eq!(
            400,
            response.status().as_u16(),
            "did not fail when: {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_data() {
    let app = spwan_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // act
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        // assert, check response
        assert_eq!(
            400,
            response.status().as_u16(),
            "did not fail when: {}",
            description
        );
    }
}
