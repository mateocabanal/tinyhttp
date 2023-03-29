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
//! fn get() -> &'static str {
//!  "Hello, World!"
//! }
//!
//! #[post("/")]
//! fn post() -> &'static str {
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
//!
//! All route functions must be included in the `Routes::new(vec![])`
//!
//! ### Diving into the route functions
//!
//! As of now, route functions can return anything that is `Into<Vec<u8>>` or a `Response`
//!
//! ```no_run
//! use tinyhttp::prelude::*;
//!
//! // Example 1: returns anything Into<Vec<u8>>
//! #[get("/")]
//! fn ex1_get() -> &'static str {
//!     "Hello World!"
//! }
//!
//! // Example 2: same as example 1, but takes a Request as an argument
//! #[get("/ex2")]
//! fn ex2_get(req: Request) -> String {
//!     let accept_header = req.get_headers().get("accept").unwrap();
//!     format!("accept header: {}", accept_header)
//! }
//!
//! // Example 3: takes a Request as an argument and returns a Response for more control
//! #[get("/ex3")]
//! fn ex3_get(req: Request) -> Response {
//!     Response::new()
//!         .status_line("HTTP/1.1 200 OK\r\n")
//!         .mime("text/plain")
//!         .body(b"Hello from response!\r\n".to_vec())
//! }

pub use tinyhttp_codegen as codegen;
pub use tinyhttp_internal as internal;

pub mod prelude {
    pub use tinyhttp_codegen::*;
    pub use tinyhttp_internal::codegen::route::*;
    pub use tinyhttp_internal::config::*;
    pub use tinyhttp_internal::request::Request;
    pub use tinyhttp_internal::response::Response;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_codegen() {
        use crate::prelude::*;

        #[get("/")]
        fn get() -> &'static str {
            "Hello Test?"
        }

        #[post("/")]
        fn post(body: Request) -> String {
            let headers = body.get_headers();
            format!(
                "Accept-Encoding: {}",
                headers.get("Accept-Encoding").unwrap()
            )
        }

        let routes = Routes::new(vec![get(), post()]);
        assert_eq!(
            false,
            routes
                .clone()
                .get_stream()
                .clone()
                .first()
                .unwrap()
                .wildcard()
                .is_some()
        );
        let request = Request::new(
            b"Hello".to_vec(),
            vec!["Accept-Encoding: gzip".to_string()],
            vec!["GET".to_string(), "/".to_string(), "HTTP/1.1".to_string()],
            None,
        );
        assert_eq!(
            b"Hello Test?".to_vec(),
            routes
                .clone()
                .get_stream()
                .first()
                .unwrap()
                .to_res(request.clone())
                .body
                .unwrap()
        );

        assert_eq!(
            b"Accept-Encoding: gzip".to_vec(),
            routes
                .clone()
                .get_stream()
                .last()
                .unwrap()
                .to_res(request.clone())
                .body
                .unwrap()
        );
    }
}
