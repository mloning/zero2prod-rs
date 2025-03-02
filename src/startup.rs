use crate::config::{Config, DatabaseConfig};
use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use secrecy::{ExposeSecret, SecretBox};
use sqlx::PgPool;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn create_db_connection_pool(config: &DatabaseConfig) -> PgPool {
    tracing::info!("Setting up (lazy) database connection ...");
    let connection_string = config.connection_string();
    PgPool::connect_lazy_with(connection_string)
}

pub struct Application {
    server: Server,
    ip: String,
    port: u16,
}

impl Application {
    pub async fn launch(config: &Config) -> Result<Application, Error> {
        tracing::info!("Building app ...");
        // set up email client
        let timeout = config.email_client.parse_timeout();
        let sender_email = config
            .email_client
            .parse_sender_email()
            .expect("could not parse sender email");

        // we move values out of config, but only have a shared reference here;
        // that's why we clone the values; an alternative would be to pass on
        // references to the values but this would requires further changes in EmailClient;
        // another alternative would be to take owernship of config in the function
        // signature, but this is undesirable because we may need config after passing
        // it to this function; we could also take ownership but clone the entire config
        // before passing it to this function, as suggested in the book, but in the latest
        // version of secrecy, SecretBox does not implement the Clone trait anymore; passing
        // a reference instead of taking ownership also seems more appropriate as we don't
        // expect this function to make any changes to config SecretBox does not implement
        // clone, so we here manually clone the secret
        let auth_token = config.email_client.auth_token.expose_secret();
        let cloned_auth_token = SecretBox::new(Box::new(auth_token.clone()));
        let email_client = EmailClient::new(
            config.email_client.base_url.clone(),
            sender_email,
            timeout,
            cloned_auth_token,
        );

        // set up database connection, with lazy connection when used for the first time
        let db_pool = create_db_connection_pool(&config.db);

        // bind to random port
        let address = format!("{}:{}", config.app.host, config.app.port);
        let listener = TcpListener::bind(address)?;
        let assigned_address = listener.local_addr().unwrap();
        let port = assigned_address.port();
        let ip = assigned_address.ip().to_string();

        // launch server
        tracing::info!("Launching server ...");
        let server = run_server(listener, db_pool, email_client)?;
        Ok(Self { server, ip, port })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        tracing::info!("App running at: {} ...", self.get_address());
        self.server.await
    }

    pub fn get_address(&self) -> String {
        format!("http://{}:{}", self.ip, self.port)
    }
}

fn run_server(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, Error> {
    tracing::info!("Launching app ...");
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            // when cloning the email client, we clone pointer to same HTTP connection pool, so we
            // can reuse open connections from the same pool across our application threads
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
