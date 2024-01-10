//! tests/health_check.rs

#[tokio::test]
async fn health_check_works() {
    spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:8000/health_check") // assert that /health_check exists
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success()); // assert that we get a 2xx response
    assert_eq!(response.content_length(), Some(0)); // assert that the body is empty
}

fn spawn_app() {
    // we could propagate the error using `?`, but there's no need as we're in a
    // test environment
    //
    // Instead, we can extract the value from the Result using .expect, or crash
    // things right here and now if we have Result::Err
    let server = zero2prod::run().expect("Failed to bind address");

    // tokio::spawn runs our server as a background process, which ensures that
    // we can write tests against the server without the spawning of the server
    // blocking any subsequent code
    tokio::spawn(server);
}
