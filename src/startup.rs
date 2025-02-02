use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use std::io::Error;
use std::net::TcpListener;

pub fn create_server(listener: TcpListener) -> Result<Server, Error> {
    let server = HttpServer::new(|| {
        App::new()
            // .route("/", web::get().to(health_check))
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
