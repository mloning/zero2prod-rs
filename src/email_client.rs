use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretBox};

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

pub struct EmailClient {
    http_client: Client,
    sender_email: SubscriberEmail,
    base_url: String,
    auth_token: SecretBox<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender_email: SubscriberEmail,
        timeout: std::time::Duration,
        auth_token: SecretBox<String>,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            sender_email,
            auth_token,
        }
    }
    pub async fn send_email(
        &self,
        receiver_email: SubscriberEmail,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender_email.as_ref(),
            to: receiver_email.as_ref(),
            subject,
            html_body,
            text_body,
        };
        self.http_client
            .post(&url)
            .json(&request_body)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretBox;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    fn make_email() -> SubscriberEmail {
        let email = SafeEmail().fake();
        SubscriberEmail::parse(email).unwrap()
    }

    fn make_subject() -> String {
        Sentence(1..2).fake()
    }

    fn make_body() -> String {
        Paragraph(1..10).fake()
    }

    fn make_email_client(base_url: String) -> EmailClient {
        let sender = make_email();
        let auth_token = SecretBox::new(Faker.fake());
        let timeout = std::time::Duration::from_millis(200);
        EmailClient::new(base_url, sender, timeout, auth_token)
    }

    #[tokio::test]
    async fn send_email_succeeds_if_email_server_responds_200() {
        // arrange
        let server = MockServer::start().await;
        let client = make_email_client(server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1) // set to expect one request
            .mount(&server)
            .await;

        // act
        let receiver_email = make_email();
        let subject = make_subject();
        let body = make_body();
        let response = client
            .send_email(receiver_email, &subject, &body, &body)
            .await;

        // assert
        // expectations from the mock server are also checked when mock objects are dropped
        assert_ok!(response);
    }

    #[tokio::test]
    async fn send_email_fails_if_email_server_responds_500() {
        // arrange
        let server = MockServer::start().await;
        let client = make_email_client(server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(500))
            .expect(1) // set to expect one request
            .mount(&server)
            .await;

        // act
        let receiver_email = make_email();
        let subject = make_subject();
        let body = make_body();
        let response = client
            .send_email(receiver_email, &subject, &body, &body)
            .await;

        // assert
        assert_err!(response);
    }

    #[tokio::test]
    async fn send_email_times_out_if_email_server_takes_too_long() {
        // arrange
        let server = MockServer::start().await;
        let client = make_email_client(server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180)))
            .expect(1) // set to expect one request
            .mount(&server)
            .await;

        // act
        let receiver_email = make_email();
        let subject = make_subject();
        let body = make_body();
        let response = client
            .send_email(receiver_email, &subject, &body, &body)
            .await;

        // assert
        assert_err!(response);
    }
}
