use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(name = "Write subscription to database", skip(form, db_pool))]
async fn write_subscription_to_db(db_pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
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
        form.email,
        form.name,
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

#[tracing::instrument(
    name = "Save subscription",
    skip(form, db_pool),  // skip attaching arguments to context of the span
    fields(  // manually add to the context of the span
        request_id = %Uuid::new_v4(),
        %form.email,
        %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let mut response = match write_subscription_to_db(&db_pool, &form).await {
        Ok(_) => HttpResponse::Ok(),
        Err(_) => HttpResponse::InternalServerError(),
    };
    response.finish()
}
