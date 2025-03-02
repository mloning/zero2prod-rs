use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::config::{read_config, DatabaseConfig};
use zero2prod::startup::{create_db_connection_pool, Application};
use zero2prod::telemetry::configure_tracing;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        let client = reqwest::Client::new();
        client
            .post(format!("{}/subscriptions", self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

/// Configure database for testing
async fn configure_db(config: &DatabaseConfig) -> PgPool {
    tracing::info!("Configuring database for testing ...");
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

/// spawn app for testing
pub async fn spwan_app() -> TestApp {
    // configure tracing only once; all other calls are skipped
    Lazy::force(&TRACING);

    // read config
    let mut config = read_config().expect("failed to read config");
    tracing::info!("Randomizing config for testing ...");
    config.db.name = Uuid::new_v4().to_string(); // randomize database name for testing
    config.app.port = 0; // use random, system assigned port

    // configure database
    configure_db(&config.db).await;

    // build server
    let app = Application::launch(&config)
        .await
        .expect("failed to build server");
    let address = app.get_address();

    // tokio::spawn spaws a new task (our server) when a new tokio runtime is launched and shuts
    // down all tasks when the runtime is stopped; tokio::test launches the new runtime
    tracing::info!("Launching test app at: {} ...", address);
    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(app.run_until_stopped());

    let db_pool = create_db_connection_pool(&config.db);
    TestApp { db_pool, address }
}
