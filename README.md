# zero2prod

Notes and annotations from the [zero2prod](https://www.zero2prod.com/) book

## 1. Introduction

- `lld` is an alternative linker to the default Rust linker, which can speed up
  compile times during development
- `cargo-watch` can run commands in sequence when files change

## 3. Sign up a new subscriber

### actix_web

- resources:
  - docs: https://actix.rs/docs/actix/
  - examples: https://github.com/actix/examples
- explanation for `#[tokio::main]`:
  - Rust does not implement an async runtime
  - Rust allows one to specify their own runtime
  - `HttpServer::run` is async
  - async means it depends on the `Future` trait
  - a `Future` needs to be polled until it resolves
    - `Future` can be thought of as lazy - until a `Future` is polled, there is
      no guarantee that it will execute to completion
  - Rust does not allow `main` to be async
    - if `main` is async, then who is going to poll `main`?
    - this seems contradictory to what's actually in the code, but this is what
      the `#[tokio::main]` macro handles - it creates the illusion that we can
      specify that `main` is async, when under the hood it's adding boilerplate
      for `tokio`s runtime
- a macro is _somewhat_ like a decorator, except that instead of augmenting a
  decorated function's inputs or output, it modifies actual code - macros are
  primarily used for code generation.

  We can evaluate the result of applying macros using the `cargo-expand` library

  e.g.

  ```bash
  $ cargo expand --bin zero2prod
  ```

  Once expanded, we can see that `main` is indeed synchronous - the `async`
  keyword has been removed

- a `Responder` is an object that can be converted to an `HttpResponse`
- `HttpResponse::Ok().finish()` will return a response with an empty body
- framework-agnostic integration tests allow for one to refactor the application
  while maintaining the same API
- `[[key]]` in `Cargo.toml` indicates we're working with an array
  - a Rust project may have only one library, but may have multiple binaries

#### Extractors

- extractors in `actix_web` extract data from incoming requests, including:
  - getting dynamic path segments
  - query parameters
  - parsing JSON-encoded request bodies
  - and more...
- the extractor we're going to use to extract data from requests that have the
  content type of `application/www-x-form-urlencoded` is `actix_web::web::Form`
  - `Form` expects a generic type, and that type must implement serde's
    `DeserializeOwned` trait

### `tokio`

- `tokio::test` spins up a new runtime for every test
- `tokio::spawn` accepts a `Future`, running it as a background task. This is
  useful when we have some async process that would block tests from being run
  were it not run as a background task, such as our server
- when a test finishes, `tokio::test` will shut down the runtime
  - any spawned tasks in the test, such as those created by `tokio::spawn`,
    are dropped, automatically cleaning up any background processes we may
    have started

### `std::net::TcpListener`

- hardcoding an address to handle requests is all good and well, but our
  integration tests quickly reveal how limited this approach is:
  - each test should be run in isolation
  - each test cannot make requests against the same port, then, because the
    server will be dropped each time other tests complete
  - we therefore need some way to dynamically get ports for each test to run
    on so that they don't interfere with each other
- Unix's `0` port will dynamically select a port - it is itself not a part that
  can be bound to - only used to find an available port
  - we can thus specify that each test use `127.0.0.1:0` to use an available
    port for the test
  - we can then pass an address explicitly to `zero2prod::run` specifying
    which address to start our server
  - this produces a new problem, however... for each test we need to know
    which port has been made available before making a request against the
    endpoint... how do we know which port the OS selected?
- so, instead of providing an address for the server to run on, we need a
  different mechanism:
  - we need to bind to a port dynamically
  - get that port
  - provide the address to the test once the port is bound
- we can do this using `TcpListener`, which binds to a socket given an address:

  1.  we create a listener by specifying that we want the OS to resolve the
      port:

      ```rust
      let listener = TcpListener::bind("127.0.0.1:0")
          .expect("Failed to bind random port");
      ```

  1.  we get the port from the listener via `TcpListener::local_addr`:

      ```rust
      let port = listener.local_addr().unwrap().port();
      ```

  1.  we provide the port to the test:

      ```rust
      let address = format("http://127.0.0.1:{}", port)
      ```

### serde

- serde doesn't do any serialisation / deserialisation of its own - crates that
  manage specific formats are responsible for the actual conversion. e.g. in
  `actix_web`, the `Form::from_request` method makes use of `serde_urlencoded`
  to parse form data in requests
- serde provides macros for `Serialize` and `Deserialize` which are added to
  structs to automate conversion between formats and data structures
