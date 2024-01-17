//! tests/health_check.rs
use std::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    let host = spawn_app();
    let endpoint = format!("{host}/health_check");

    let client = reqwest::Client::new();

    let response = client
        .get(endpoint) // assert that /health_check exists
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success()); // assert that we get a 2xx response
    assert_eq!(response.content_length(), Some(0)); // assert that the body is empty
}

pub mod subscribe {
    use super::spawn_app;

    #[tokio::test]
    async fn returns_200_for_valid_form_data() {
        let address = spawn_app();
        let client = reqwest::Client::new();

        let body = "name=Jo%20Soap&email=josoap@example.com";
        let response = client
            .post(format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(response.status().as_u16(), 200);
    }

    #[tokio::test]
    async fn returns_400_for_invalid_form_data() {
        let test_cases = [
            ("name=Jo%20Soap", "email is required"),
            ("email=josoap@example.com", "name is required"),
            ("", "name and email are required"),
        ];

        for (body, error_message) in test_cases.iter() {
            let address = spawn_app();
            let client = reqwest::Client::new();

            let response = client
                .post(format!("{}/{}", &address, "subscriptions"))
                .header("Content-Type", "application/x-www-form-urlencoded")
                .body(*body)
                .send()
                .await
                .expect("Failed to execute request");

            assert_eq!(
                response.status().as_u16(),
                400,
                "Api did not fail with 400 Bad Request when payload was {}",
                error_message
            );
        }
    }
}

/// Spin up an instance of the application, and returns its address
/// i.e. http://localhost:XXXX
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    // we could propagate the error using `?`, but there's no need as we're in a
    // test environment
    //
    // Instead, we can extract the value from the Result using .expect, or crash
    // things right here and now if we have Result::Err
    //
    // Port 0 at the OS-level will scan for available ports. This allows each test
    // to be run at its own isolated port
    //
    // This, however, means we can no longer rely on the hardcoded port in the
    // test... we need some way to get the port allocated for each specific
    // test-run
    let server = zero2prod::startup::run(listener).expect("Failed to bind address");

    // tokio::spawn runs our server as a background process, which ensures that
    // we can write tests against the server without the spawning of the server
    // blocking any subsequent code
    //
    // tokio::test works in tandem with tokio::spawn:
    //  - tokio::test spins up a runtime for each test at the beginning of the
    //      test
    //  - tokio::spawn spawns a thread to run a process in the background
    //  - when the test is finished, the runtime is shut down
    //  - when a runtime is shutdown, all associated tasks that have been spawned
    //      using tokio::spawn are dropped
    //
    // This means that every time spawn_app is run, the server will be shut down
    // when the test is finished - no clean up code required
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
