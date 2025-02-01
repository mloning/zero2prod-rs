use std::io::Error;
use zero2prod_rs::create_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    create_server()?.await
}
