//! tests/health_check.rs
use std::net::TcpListener;
use uuid::Uuid;

use zero2prod::configuration;

use sqlx::{Connection, Executor, PgConnection};

pub struct TestApp {
    pub address: String,
    pub db_pool: sqlx::PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let TestApp { address: host, .. } = spawn_app().await;
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
    use sqlx::{Connection, PgConnection};
    use zero2prod::configuration::get_configuration;

    use super::spawn_app;
    use super::TestApp;

    #[tokio::test]
    async fn returns_200_for_valid_form_data() {
        let TestApp { address, db_pool } = spawn_app().await;
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

        let entity = sqlx::query!("SELECT email, name FROM subscriptions")
            .fetch_one(&db_pool)
            .await
            .expect("Failed to fetch saved subscription");

        assert_eq!(entity.email, "josoap@example.com");
        assert_eq!(entity.name, "Jo Soap");
    }

    #[tokio::test]
    async fn returns_400_for_invalid_form_data() {
        let test_cases = [
            ("name=Jo%20Soap", "email is required"),
            ("email=josoap@example.com", "name is required"),
            ("", "name and email are required"),
        ];

        for (body, error_message) in test_cases.iter() {
            let TestApp { address, .. } = spawn_app().await;
            let config = get_configuration().expect("Failed to get config");
            let connection_string = config.database.connection_string();
            let client = reqwest::Client::new();

            PgConnection::connect(&connection_string)
                .await
                .expect("Unable to connect to Postgres");

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
async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let mut config = configuration::get_configuration().expect("Unable to get configuration");

    config.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&config.database).await;

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
    let server =
        zero2prod::startup::run(listener, connection_pool.clone()).expect("Failed to bind address");

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

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: connection_pool,
    }
}

async fn configure_database(db_config: &configuration::DatabaseSettings) -> sqlx::PgPool {
    // connect to Postgres
    // PgConnection::Connect requires Connect to be in scope
    let mut connection = PgConnection::connect(&db_config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    // create database
    connection
        // PgConnection::execute requires Executor to be in scope
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_config.database_name).as_str())
        .await
        .expect("Unable to create database");

    // connect to database
    let connection_pool = sqlx::PgPool::connect(&db_config.connection_string())
        .await
        .expect("Failed to connect to PostGres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}
