//! # tinyhttp
//!
//! `tinyhttp` is a fast, multi-threaded http server with procedural macros
//! to make life easier.
//!
//! You can also combine examples, e.g with a mount point and custom routes

//! # Example 1: Using a mount point
//! ```no_run
//! use std::net::TcpListener;
//! use tinyhttp::internal::config::*;

//! fn main() {
//!   let socket = TcpListener::bind(":::9001").unwrap();
//!   let mount_point = ".";
//!   let config = Config::new().mount_point(mount_point);
//!   let http = HttpListener::new(socket, config);
//!
//!   http.start();
//! }
//! ```

//! # Example 2: Using proc macros

//! ```no_run
//!
//! /// Functions marked with the get macro must return with a type of Into<Vec<u8>>
//! use std::net::TcpListener;
//! use tinyhttp::prelude::*;
//! #[get("/")]
//! fn get(_body: Request) -> &'static str {
//!  "Hello, World!"
//! }
//!
//! /// Functions marked with the post macro must explicitly return a Vec<u8>, but many types are
//! /// compatible with Into<Vec<u8>>. Must manually do it though.
//!
//! #[post("/")]
//! fn post(body: Request) -> &'static str {
//!   "Hello, there!"
//! }
//!
//! fn main() {
//!   let socket = TcpListener::bind(":::80").unwrap();
//!   let routes = Routes::new(vec![get(), post()]);
//!   let config = Config::new().routes(routes);
//!   let http = HttpListener::new(socket, config);
//!
//!   http.start();
//! }
//! ```

pub use tinyhttp_codegen as codegen;
pub use tinyhttp_internal as internal;

pub mod prelude {
    pub use tinyhttp_codegen::*;
    pub use tinyhttp_internal::codegen::route::{GetRoute, PostRoute};
    pub use tinyhttp_internal::config::*;
    pub use tinyhttp_internal::request::Request;
}

