use std::net::TcpListener;

fn spwan_app() -> String {
    let address = "127.0.0.1";
    let listener = TcpListener::bind(format!("{}:0", address)).expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port(); // port assigned by OS
    let server = zero2prod_rs::create_server(listener).expect("failed to create server");
    // tokio::spawn spaws a new task (our server) when a new tokio runtime is launched and shuts
    // down all tasks when the runtime is stopped; tokio::test launches the new runtime
    let _ = tokio::spawn(server);
    format!("http://{}:{}", address, port)
}

#[tokio::test]
async fn test_health_check() {
    // arange, start app and create a client
    let address = spwan_app();
    let client = reqwest::Client::new();

    // act, send request
    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request");

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(Some(0), response.content_length()); // no body
}
