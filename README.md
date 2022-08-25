# HTTP SERVER

[![codecov](https://codecov.io/gh/mateocabanal/tinyhttp/branch/main/graph/badge.svg?token=LH8YSHNKRF)](https://codecov.io/gh/mateocabanal/tinyhttp)
[![Rust](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml/badge.svg)](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml)
![Crates.io](https://img.shields.io/crates/d/tinyhttp?color=purple&logo=cargo&style=for-the-badge)

![Alt](https://repobeats.axiom.co/api/embed/eb3b94060f5f66f0fc1be7ccbc7f581d10c7ca34.svg "Repobeats analytics image")

## [Live Demo](https://mateo-tinyhttp.herokuapp.com/)

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
