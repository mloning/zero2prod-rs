use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretBox};

#[derive(serde::Serialize)]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
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
        auth_token: SecretBox<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
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
            from: self.sender_email.as_ref().to_owned(),
            to: receiver_email.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_body: html_body.to_owned(),
            text_body: text_body.to_owned(),
        };
        self.http_client
            .post(&url)
            .json(&request_body)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .send()
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::SecretBox;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_sends_request_to_base_url() {
        let server = MockServer::start().await;
        let email = SafeEmail().fake();
        let sender = SubscriberEmail::parse(email).unwrap();
        let auth_token = SecretBox::new(Faker.fake());
        let client = EmailClient::new(server.uri(), sender, auth_token);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let recipient = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        let _ = client
            .send_email(recipient, &subject, &content, &content)
            .await;
    }
}
