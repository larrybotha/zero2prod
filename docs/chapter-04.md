# Chapter 4 - Telemetry

Telemetry helps dealing with unknown unknowns - scenarios that are difficult to
reproduce outside of the production environment:

- spikes in traffic
- multiple component failures, e.g. SQL transaction hanging while the db is
  going through an update / recovery
- changes to policies in the application
- server resources diminishing due to long-running processes, e.g. memory leaks,
  disk availability

The goal is to build a sufficiently observable application:

- collect high-quality telemetry data
- using tools that allow for analysis of the data

_Instrumentation_ is the process of adding code to software to collect telemetry
data.

## Logging

The `log` crate uses the facade pattern. It makes no assumption on how logs are
processed. This is left to a `Log` trait for the user to specify when
configuring logging.

This trait is where one defines if emitted logs should be printed to stdout, a
file, or somewhere else.

`env_logger::Logger` implements `Log`, and prints logs to `stdout`

- user requests should be investigable by evaluating logs - ensure there is
  sufficient information to trace a user's feedback to a log
  - this may have privacy implications - logging personally identifiable
    information should be avoided
- differentiating different requests from each other is crucial - multiple
  requests may result in a non-linear set of logs. Request / correlation IDs
  in logs are one way of being able to correlate related logs
  - using a uuid can be useful for this
- manually adding IDs to requests doesn't scale:
  - we'd need to add it to every request
  - the middleware has no idea that the ID exists
  - what about 3rd party libraries - how we include request IDs in those traces?
- a better alternative to logging is _tracing_

## Tracing

Tracing expands on on logging by allowing libraries and applications to record
information about time and causation.

A span in tracing is superior to a single logged event for observability:

- a span has a start and end, as opposed to a log being a singular event at a
  point in time
- a span is a nested tree structure, whereas a log is flat

The `tracing` crate has a `log` feature which will ensure that all traces emit a
log event, which can then be picked up by the `log` crate.

`tracing` allows for providing structured data - we don't use interpolation, as
with `log::info!`. Instead, `tracing::info_span!` allows for specifying
information as key-value pairs.

To create a span:

- create a span using the `tracing::info_span!` macro, or for whichever log level is
  appropriate
- get a guard for the span by _stepping into_ the span:
  - in sync environments it's safe to use `my_spawn.enter()`
  - in async environments... TBD
- `.enter()` returns an instance of `Entered`. Evaluating `impl Drop for Entered`
  shows that if the `log` feature is enabled for `tracing`, that a trace-level
  log will be emitted

  See the `if_log_enabled!` macro in `Span::do_exit`

- as long as the guard isn't dropped, all downstream spans and logs will be
  registered as children of the span
- using `tracing::info!` etc. to add events to the span within the scope

The span's lifetime is delimited the following:

- `-->` span is entered
- `<--` span is exited
- `--` span is closed

Spans can be entered and exited multiple times - i.e. they can be paused, e.g.
across threads, async tasks, etc.

## Links and resources

- [Formal methods / TLA+](https://lamport.azurewebsites.net/tla/formal-methods-amazon.pdf)
- [log - defactor logging crate for Rust applications](https://crates.io/crates/log)
- [tracing](https://crates.io/crates/tracing)
