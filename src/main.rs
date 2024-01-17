//! src/main.rs

use zero2prod::startup;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Create a socket that is bound to 8000 on the host
    // This listener is the server, and client requests made to localhost:8000 will
    // be handled by this server
    let listener = std::net::TcpListener::bind("127.0.0.1:8000")?;

    startup::run(listener)?.await
}
