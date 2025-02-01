fn spwan_app() {
    let server = zero2prod_rs::create_server().expect("failed to create server");
    let _ = tokio::spawn(server);
}

#[tokio::test]
async fn test_health_check() {
    // arange, start app and create a client
    spwan_app();
    let client = reqwest::Client::new();

    // act, send request
    let response = client
        .get("http://127.0.0.1:8000/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(Some(0), response.content_length()); // no body
}
