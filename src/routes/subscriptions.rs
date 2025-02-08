use actix_web::{web, HttpResponse};
use chrono::Utc;
use log;
use sqlx;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Handling request to save subscription",
        %request_id,
        subscriber_name = %form.name,
        subscriber_email = %form.email
    );
    let _request_span_guard = request_span.enter();

    let id = Uuid::new_v4();
    let subscribed_at = Utc::now();

    let query_span = tracing::info_span!("Saving new subscription to the database", %id);
    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        id,
        form.email,
        form.name,
        subscribed_at
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await;

    let mut response = match result {
        Ok(_) => {
            tracing::info!("Saved new subscription with id: {}", id);
            HttpResponse::Ok()
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError()
        }
    };
    response.finish()
}
