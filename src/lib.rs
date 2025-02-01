use std::io::Error;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().finish()
}

pub fn create_server() -> Result<Server, Error> {
    let server = HttpServer::new(|| {
        App::new()
            // .route("/", web::get().to(health_check))
            .route("/health_check", web::get().to(health_check))
    })
    .bind("127.0.0.1:8000")?
    .run();
    Ok(server)
}
