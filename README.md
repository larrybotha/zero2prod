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
