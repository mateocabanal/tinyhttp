# HTTP SERVER

[![codecov](https://codecov.io/gh/mateocabanal/tinyhttp/branch/main/graph/badge.svg?token=LH8YSHNKRF)](https://codecov.io/gh/mateocabanal/tinyhttp)
[![Rust](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml/badge.svg)](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml)
![Crates.io](https://img.shields.io/crates/d/tinyhttp?color=purple&logo=cargo&style=for-the-badge)

![Alt](https://repobeats.axiom.co/api/embed/eb3b94060f5f66f0fc1be7ccbc7f581d10c7ca34.svg "Repobeats analytics image")

## [Live Demo](https://mateo-tinyhttp.fly.dev)

#### The "tinyhttp" crate contains none of the internal code due to the procedural macro crate depending on data types on the internal crate.


Speedy HTTP server built purely in Rust. Comes with built-in GZIP compression and HTTPS support.

Uses procedural macros for easy API building.

### Performance
On a Raspberry Pi 4 with ethernet, tinyhttp is able to serve around 15000 requests per second

This was tested with [go-wrk](https://github.com/tsliwowicz/go-wrk)

### Examples

Example :
```rust
use std::net::TcpListener;
use tinyhttp::prelude::*;

#[get("/")]
fn get() -> &'static str {
  "Hello, World!"
}

#[post("/")]
fn post() -> &'static str {
  "Hi, there!"
}

// Example 1: Can return anything that implements Into<Vec<u8>>
#[get("/")]
fn ex1_get() -> &'static str {
  "Hello World!"
}

// Example 2: same as example 1, but takes a Request as an argument
#[get("/ex2")]
fn ex2_get(req: Request) -> String {
    let accept_header = req.get_headers().get("accept").unwrap();
    format!("accept header: {}", accept_header)
}

// Example 3: takes a Request as an argument and returns a Response for more control
#[get("/ex3")]
fn ex3_get(req: Request) -> Response {
    Response::new()
        .status_line("HTTP/1.1 200 OK\r\n")
        .mime("text/plain")
        .body(b"Hello from response!\r\n".to_vec())
}

fn main() {
  let socket = TcpListener::bind(":::9001").unwrap();
  let routes = Routes::new(vec![get(), post()]);
  let config = Config::new().routes(routes);
  let http = HttpListener::new(socket, config);

  http.start();
}
```
