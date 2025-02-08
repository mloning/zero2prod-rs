use actix_web::{web, HttpResponse};
use chrono::Utc;
use log;
use sqlx;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    log::info!(
        "Handling request to save subscription with form: {:#?} ...",
        form
    );
    let id = Uuid::new_v4();
    let subscribed_at = Utc::now();
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
    .await;

    let mut response = match result {
        Ok(_) => {
            log::info!("Saved new subscription with id: {}", id);
            HttpResponse::Ok()
        }
        Err(e) => {
            log::error!("Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError()
        }
    };
    response.finish()
}
