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
async fn health_check_returns_200_and_no_body() {
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
    assert_eq!(200, response.status().as_u16());
    assert_eq!(Some(0), response.content_length()); // no body
}

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    // arange, start app and create a client
    let address = spwan_app();
    let client = reqwest::Client::new();

    // act, send request
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_data() {
    // arange, start app and create a client
    let address = spwan_app();
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
            .post(format!("{}/subscriptions", address))
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
