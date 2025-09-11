use actix_web::{HttpResponse, Responder, web};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info, instrument};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateData {
    name: String,
    email: String,
}

#[instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        request_id = %Uuid::new_v4(),
        subscriber.email = %data.email,
        subscriber.person_name = %data.name,
    )
)]
pub async fn subscribe(db_pool: web::Data<PgPool>, data: web::Form<CreateData>) -> impl Responder {
    match insert_subscriber(&db_pool, data.into_inner()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[instrument(
    name = "Saving new subscriber details in the database",
    skip_all,
    fields(subscription_id)
)]
pub async fn insert_subscriber(db_pool: &PgPool, data: CreateData) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    tracing::Span::current().record("subscription_id", id.to_string());

    sqlx::query!(
        "INSERT INTO subscriptions (id, email, person_name, subscribed_at) VALUES ($1, $2, $3, $4)",
        id,
        data.email,
        data.name,
        Utc::now(),
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        error!("Failed to execute query: {e:?}");
        e
    })?;

    info!("New subscriber details have been saved");
    Ok(())
}
