use std::net::TcpListener;

use actix_web::{App, HttpResponse, HttpServer, Responder, dev::Server, web};
use serde::Deserialize;

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, Deserialize)]
struct SubscribeData {
    name: String,
    email: String,
}

async fn subscribe(data: web::Form<SubscribeData>) -> impl Responder {
    println!("New subscription from {} [{}]", data.name, data.email);
    HttpResponse::Ok()
}
