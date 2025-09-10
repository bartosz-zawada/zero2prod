use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SubscribeData {
    name: String,
    email: String,
}

pub async fn subscribe(
    form: web::Form<SubscribeData>,
    connection: web::Data<PgPool>,
) -> impl Responder {
    match sqlx::query!(
        "INSERT INTO subscriptions (id, email, person_name, subscribed_at) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok(),
        Err(e) => {
            println!("Failed to execute query: {e}");
            HttpResponse::InternalServerError()
        }
    }
}
