use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};

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
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    // Every time HttpServer::new's is called, actix will create its own worker
    // for the application instance. HttpServer::new's closure expects us to return
    // an instance of App
    // If PgPool implemented Clone, we could pass connection into app_data
    // without a problem, but because _it doesn't_, the first worker would take
    // ownership of 'connection', and no other workers would have access to the
    // connection
    // We somehow need to allow multiple processes, or threads, access to the same
    // connection... how about Arc to create a shared atomic reference counter?
    //let connection = Arc::new(connection);
    // Arc seems, to work, but web::Data is built specifically for this
    let db_pool = web::Data::new(db_pool);

    let server = HttpServer::new(move || {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // register the connection as part of the application state
            .app_data(db_pool.clone())
    })
    // We can bind to a specific address on the host, or...
    //.bind("127.0.0.1:8000")?
    // we can listen to a TcpListener
    .listen(listener)?
    .run(); // run is asynchronous, so we need to make `main` async, too

    Ok(server)
}
