use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

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
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber = match NewSubscriber::try_from(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    let mut response = match write_subscriber_to_db(&db_pool, &subscriber).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    };
    response.finish()
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
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        id,
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
        subscribed_at
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to write new subscription to database: {:?}", e);
        e
    })?; // using `?` to return early if error
    Ok(())
}
