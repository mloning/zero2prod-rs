use crate::email_client::EmailClient;
use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn create_server(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, Error> {
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
