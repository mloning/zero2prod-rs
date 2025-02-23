use crate::helpers::spwan_app;

#[tokio::test]
async fn health_check_returns_200_and_no_body() {
    // arange, start app and create a client
    let app = spwan_app().await;
    let client = reqwest::Client::new();

    // act, send request
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(200, response.status().as_u16());
    assert_eq!(Some(0), response.content_length()); // no body
}
