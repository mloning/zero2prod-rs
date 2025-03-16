use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::email_client::EmailClient;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        let subscriber = Self { email, name };
        Ok(subscriber)
    }
}

#[tracing::instrument(
    name = "Save subscription",
    skip(form, db_pool),  // skip attaching arguments to context of the span
    fields(  // manually add to the context of the span
        %form.email,
        %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let subscriber = match NewSubscriber::try_from(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    if write_subscriber_to_db(&db_pool, &subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    };

    if send_confirmation_email(&email_client, subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };
    HttpResponse::Ok().finish()
}
#[tracing::instrument(name = "Send confirmation email", skip(subscriber, email_client))]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://domain.com/subscriptions/confirm";
    let subject = "Welcome!";
    let text_body = format!(
        "Welcome! Click: {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(subscriber.email, subject, &html_body, &text_body)
        .await
}

#[tracing::instrument(name = "Write subscriber to database", skip(subscriber, db_pool))]
async fn write_subscriber_to_db(
    db_pool: &PgPool,
    subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    let subscribed_at = Utc::now();
    tracing::info!(
        "Writing subscription to the database: {}, {}",
        id,
        subscribed_at
    );
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
        "#,
        id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        subscribed_at,
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to write new subscription to database: {:?}", e);
        e
    })?; // using `?` to return early if error
    Ok(())
}
