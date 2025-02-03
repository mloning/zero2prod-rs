use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
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
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("failed to execute query: {}", e);
            HttpResponse::InternalServerError()
        }
    };
    response.finish()
}
