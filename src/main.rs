//! src/main.rs

use sqlx::PgPool;
use zero2prod::configuration;
use zero2prod::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");
    let pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to database");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    // Create a socket that is bound to a port on the host
    // This listener is the server, and client requests made to localhost:8000 will
    // be handled by this server
    let listener = std::net::TcpListener::bind(address)?;

    startup::run(listener, pool)?.await
}
