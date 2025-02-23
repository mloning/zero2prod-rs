use crate::helpers::spwan_app;

#[tokio::test]
async fn subscribe_returns_200_for_valid_data() {
    // arange, start app and create a client
    let app = spwan_app().await;
    let client = reqwest::Client::new();

    // act, send request
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

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
            .post(format!("{}/subscriptions", app.address))
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

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_data() {
    let app = spwan_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    for (body, description) in test_cases {
        // act
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        // assert, check response
        assert_eq!(
            400,
            response.status().as_u16(),
            "did not fail when: {}",
            description
        );
    }
}
