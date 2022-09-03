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
                .is_args()
        );
        assert_eq!(
            b"Hello Test?".to_vec(),
            routes
                .clone()
                .get_stream()
                .first()
                .unwrap()
                .get_body()
                .unwrap()()
        );

        let request = Request::new(
            b"Hello".to_vec(),
            vec!["Accept-Encoding: gzip".to_string()],
            vec!["GET".to_string(), "/".to_string(), "HTTP/1.1".to_string()],
            None,
        );

        assert_eq!(
            b"Accept-Encoding: gzip".to_vec(),
            routes
                .clone()
                .get_stream()
                .last()
                .unwrap()
                .post_body_with()
                .unwrap()(request)
        );

        assert_eq!(
            None,
            routes.clone().get_stream().last().unwrap().post_body()
        );
    }
}
