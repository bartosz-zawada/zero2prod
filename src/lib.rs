use actix_web::{App, HttpResponse, HttpServer, Responder, dev::Server, web::get};

pub fn run() -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health_check", get().to(health_check)))
        .bind("127.0.0.1:8000")?
        .run();

    Ok(server)
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
