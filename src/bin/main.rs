use std::{io::Error, net::TcpListener};
use zero2prod_rs::create_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let address = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", address)).expect("failed to bind random port");
    create_server(listener)?.await
}
