# HTTP SERVER

[![codecov](https://codecov.io/gh/mateocabanal/tinyhttp/branch/main/graph/badge.svg?token=LH8YSHNKRF)](https://codecov.io/gh/mateocabanal/tinyhttp)
[![Rust](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml/badge.svg)](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml)
![Crates.io](https://img.shields.io/crates/d/tinyhttp?color=purple&logo=cargo&style=for-the-badge)

#### The "tinyhttp" crate contains none of the internal code due to the procedural macro crate depending on data types on the internal crate.

Speedy HTTP server built purely in Rust. Comes with built-in GZIP compression and HTTPS support.

Uses procedural macros for easy API building.

tinyhttp also supports async, however it is disabled by default.
Enable the "async" feature to enable async.

### Performance
On a Raspberry Pi 4 with ethernet, tinyhttp is able to serve around 15000 requests per second

This was tested with [go-wrk](https://github.com/tsliwowicz/go-wrk)

### Examples

Blocking Example :
```rust
use std::net::TcpListener;
use tinyhttp::prelude::*;

fn main() {
  let socket = TcpListener::bind(":::9001").unwrap();
  let routes = Routes::new(vec![get(), post()]);
  let config = Config::new().routes(routes);
  let http = HttpListener::new(socket, config);

  http.start();
}

#[get("/")]
fn get() -> &'static str {
  "Hello, World!"
}

#[post("/")]
fn post(body: Request) -> &'static str {
  "Hi, there!"
}
```

Async Example:
```rust
use std::net::TcpListener;
use tinyhttp::prelude::*;

// Can replace tokio with any other async executor
#[tokio::main]
async fn main() {
  let socket = TcpListener::bind(":::9001").unwrap();
  let routes = Routes::new(vec![get(), post()]);
  let config = Config::new().routes(routes);
  let http = HttpListener::new(socket, config);

  http.start().await;
}

#[get("/")]
fn get(_req: Request) -> &'static str {
  "Hello, World!"
}

#[post("/")]
fn post(_req: Request) -> &'static str {
  "Hi, there!"
}
