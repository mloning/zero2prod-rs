use crate::domain::SubscriberEmail;
use reqwest::Client;

pub struct EmailClient {
    http_client: Client,
    sender_email: SubscriberEmail,
    base_url: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender_email: SubscriberEmail) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender_email,
        }
    }
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
