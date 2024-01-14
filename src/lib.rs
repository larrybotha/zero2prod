use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct FormData {
    name: String,
    email: String,
}

async fn subscriptions(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

// Attempt 1:
//  We use an async function returning a Future. src/main.rs will need to await the
//  Future to handle requests
//pub async fn run() -> Result<(), std::io::Error> {
//    HttpServer::new(|| App::new().route("/health_check", web::get().to(health_check)))
//        .bind("127.0.0.1:8000")?
//        .run() // run is asynchronous, so we need to make `main` async, too
//        .await
//}

// Attempt 2:
//  We use return a Result of the server. This allows us to opt into using a Future
//  src/main.rs, but allow for us to call `run` as a background task in tests
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscriptions))
    })
    // We can bind to a specific address on the host, or...
    //.bind("127.0.0.1:8000")?
    // we can listen to a TcpListener
    .listen(listener)?
    .run(); // run is asynchronous, so we need to make `main` async, too

    Ok(server)
}
