# HTTP SERVER

![Rust](https://github.com/mateocabanal/tinyhttp/actions/workflows/rust.yml/badge.svg?branch=main)
![Crates.io](https://img.shields.io/crates/d/tinyhttp?color=purple&logo=cargo&style=for-the-badge)

#### The "tinyhttp" crate contains none of the internal code due to the procedural macro crate depending on data types on the internal crate.

Speedy HTTP server built purely in Rust. Comes with built-in GZIP compression and HTTPS support.

Uses procedural macros for easy API building.

tinyhttp also supports async, however it is disabled by default.
Enable the "async" feature to enable async.

Example 1:

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
fn get(_body: Request) -> &'static str {
  "Hello, World!"
}

#[post("/")]
fn post(body: Request) -> &'static str {
  "Hi, there!"
}
```