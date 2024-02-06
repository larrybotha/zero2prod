# Chapter 3 - Sign up a new subscriber

## actix_web

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
- `App::app_data` can be used to pass application data around, making it
  accessible to requests
  - e.g. if you want to share a connection to a database, it can be passed to
    `App::app_data`
  - because we instantiate the app _inside_ `HttpServer::new`s closure, we will
    get a new `App` instance every time a new server is spawned - i.e. for
    each worker that `actix` spawns
  - Everything inside `app_data` needs to be clonable, so that it's available
    within each process
  - `PgConnection` does not implement `Clone`, so it can't be passed to
    `.app_data` as-is. How can we make a single instance available across
    threads? By using `std::sync::Arc`. `Arc` is _always_ clonable, regardless
    of what it contains
  - But we can do better than that... `.app_data` can also receive an object
    wrapped in `actix_web::web::Data`, which is itself a wrapper for `Arc`. If
    data is wrapped in `Data` when passed to `.app_data`, the data becomes an
    extractor - the data can be extracted from each request
    - `web::Data` has `.get_ref` method, which behaves similarly to `Arc::as_ref`
      provide access to the shared internal value

## Extractors

- extractors in `actix_web` extract data from incoming requests, including:
  - getting dynamic path segments
  - query parameters
  - parsing JSON-encoded request bodies
  - and more...
- the extractor we're going to use to extract data from requests that have the
  content type of `application/www-x-form-urlencoded` is `actix_web::web::Form`
  - `Form` expects a generic type, and that type must implement serde's
    `DeserializeOwned` trait

## `tokio`

- `tokio::test` spins up a new runtime for every test
- `tokio::spawn` accepts a `Future`, running it as a background task. This is
  useful when we have some async process that would block tests from being run
  were it not run as a background task, such as our server
- when a test finishes, `tokio::test` will shut down the runtime
  - any spawned tasks in the test, such as those created by `tokio::spawn`,
    are dropped, automatically cleaning up any background processes we may
    have started

## `std::net::TcpListener`

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

## serde

- serde doesn't do any serialisation / deserialisation of its own - crates that
  manage specific formats are responsible for the actual conversion. e.g. in
  `actix_web`, the `Form::from_request` method makes use of `serde_urlencoded`
  to parse form data in requests
- serde provides macros for `Serialize` and `Deserialize` which are added to
  structs to automate conversion between formats and data structures

## database

Choosing a database can be difficult, but the following are useful guidelines:

- if you're not sure what you need, start with a relational db as they are good
  for general-purpose use, and scale well initially
- compile-time errors vs runtime errors are a good consideration with a language
  like Rust. Some ORMs will only error at runtime - ideally we'd want to be
  notified at compile-time that we've done something wrong so that we expose
  our users to fewer issues
- some ORMs use DSLs, while others require raw SQL:
  - raw SQL may be less convenient, but it's portable
  - DSLs may be convenient, but you're tied into the API, and it requires
    learning and time investment
- some ORMs offer async support, others not. If your application is async,
  prefer an async ORM, as there are gotchas when attempting to mix the two
  approaches - see pg54

We're choosing [`sqlx`](https://docs.rs/sqlx/latest/sqlx/) for this project. In
addition to the application support, `sqlx` has a CLI which is useful for
migrations:

```bash
$ cargo install --version="~0.7" sqlx-cli \
  --no-default-features --features rustls,postgres
$ sqlx --version
```

- database constraints are useful, but they come at the cost of the database
  having to perform validations before performing any writes
  - before attempting to optimise constraints out of the database, ensure that
    performance is actually a problem
- `sqlx` runs asynchronously, but it doesn't allow multiple queries to be run
  concurrently using the same database connection

  To enforce this, `sqlx::execute` requires that a connection be passed in as
  a mutable reference.

  Why a mutable reference? Rust only allows a single mutable reference to
  exist at a time - i.e. a mutable reference is a _unique_ reference.
  `sqlx::execute` is guaranteed by Rust that the given connection cannot be used
  elsewhere concurrently

- requiring that `sqlx::execute` requires a single mutable connection seems
  probalematic though... how do we allow an arbitrary number of connections to
  be executed at the same time?

  For this, `sqlx` has implemented its own pooling strategy, and with the
  Postgres implementation we get a `PgPool` struct

- the database can be thought of as a global variable - multiple tests
  augmenting it can have unintended consequences. To isolate tests from each
  other, one can:
  - wrap every test inside a transaction, assert within the transaction, and
    then roll the transaction back, or
  - spin up a new database for each integration test. To do this, for every
    test, we need to:
    - create a new database name, which we use `uuid::Uuid::new_v4` to do
    - get a connection string for Postgres _without_ the database name
    - get a connection to Postgres
    - create the database given the generated name
    - obtain a connection pool to the new database using the full connection
      string
    - run any migrations on the database
    - return the connection string to be used in the tests
- `PgConnection::connect` requires `sqlx::Connect` to be in scope
- `PgConnection::connect::execute` requires `sqlx::Executor` to be in scope

## links and resources

- async/await:
  - [async/await announcement](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html#zero-cost-futures)
  - [Zero-cost Async IO](https://www.youtube.com/watch?v=skos4B5x7qE)
  - [Futures Explained](https://github.com/verhovsky/books-futures-explained/tree/master)
- [simd-json](https://docs.rs/simd-json/0.3.18/simd_json/index.html)
- [serde::serialize_seq](https://github.com/serde-rs/json/blob/4354fc3eb2232ee0ba9a9a23acce107a980a6dc0/src/ser.rs#L318)
  - Context, pg68:
    > For example, if you were adding support for JSON serialisation, your
    > serialize_seq implementation would output an opening square bracket [ and
    > return a type which can be used to serialize sequence elements.
