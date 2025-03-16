use crate::helpers::{find_links, spwan_app};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    // arange, start app and create a client
    let app = spwan_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // act, send request
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.post_subscription(body.into()).await;

    // assert, check response
    assert!(response.status().is_success()); // 200 status
    assert_eq!(200, response.status().as_u16());

    let subscription = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch data from database");

    assert_eq!(subscription.name, "le guin");
    assert_eq!(subscription.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_returns_400_for_missing_data() {
    // arange, start app and create a client
    let app = spwan_app().await;

    // TODO move test cases into parametrized fixture, using rtest crate
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (body, error_message) in test_cases {
        // act, send request
        let response = app.post_subscription(body.into()).await;

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

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_data() {
    let app = spwan_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // act
        let response = app.post_subscription(body.into()).await;

        // assert, check response
        assert_eq!(
            400,
            response.status().as_u16(),
            "did not fail when: {}",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_for_valid_data() {
    // arange, start app and create a client
    let app = spwan_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // act, send request
    app.post_subscription(body.into()).await;

    // assert, check response
    // mock asserts before drop
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_with_a_link() {
    // arange, start app and create a client
    let app = spwan_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // act, send request
    app.post_subscription(body.into()).await;

    // assert, check response
    let request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
    let html_links = find_links(body["HtmlBody"].as_str().unwrap());
    let text_links = find_links(body["TextBody"].as_str().unwrap());
    assert_eq!(html_links.len(), 1);
    assert_eq!(text_links.len(), 1);
    assert_eq!(html_links[0].as_str(), text_links[0].as_str());
}
