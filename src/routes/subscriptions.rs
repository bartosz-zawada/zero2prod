use actix_web::{HttpResponse, Responder, web};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubscribeData {
    name: String,
    email: String,
}

pub async fn subscribe(data: web::Form<SubscribeData>) -> impl Responder {
    println!("New subscription from {} [{}]", data.name, data.email);
    HttpResponse::Ok()
}
