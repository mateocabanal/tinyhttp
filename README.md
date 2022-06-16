# HTTP SERVER [![Rust](https://github.com/Yourlitdaddy/tinyhttp/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/Yourlitdaddy/tinyhttp/actions/workflows/rust.yml)

#### This repo contains none of the internal code due to the procedural macro crate depending on data types on the internal crate.

#### All internal code is now [HERE](https://github.com/yourlitdaddy/tinyhttp-internal)

Speedy HTTP server built purely in Rust. Comes with built-in GZIP compression and HTTPS support.

Uses procedural macros for easy API building.



Example 1:
```rust
use std::net::TcpListener;
use tinyhttp::internal::config::*;
use tinyhttp::codegen::*;

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
fn post(body: Request) -> Vec<u8> {
  "Hi, there!".into()
}
