use env_logger::Env;
use sqlx::PgPool;
use std::{io::Error, net::TcpListener};
use zero2prod::config::read_config;
use zero2prod::startup::create_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = read_config().expect("failed to read config");

    let address = format!("127.0.0.1:{}", config.port);
    let listener = TcpListener::bind(address).expect("failed to bind random port");

    let connection_string = config.database.connection_string();
    let db_pool = PgPool::connect(&connection_string)
        .await
        .expect("failed to connect to database");

    create_server(listener, db_pool)?.await
}
