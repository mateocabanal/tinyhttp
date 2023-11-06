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
//!     let headers = req.get_headers();
//!     let accept_header = headers.get("accept").unwrap();
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

#![allow(clippy::needless_doctest_main)]

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
    use std::{sync::OnceLock, thread, time::Duration};

    static HTTP_ENABLED: OnceLock<bool> = OnceLock::new();

    use crate::prelude::*;

    fn setup_http_server() -> Result<(), Box<dyn std::error::Error>> {
        let sock = std::net::TcpListener::bind("127.0.0.1:23195");
        if sock.is_err() {
            return Ok(());
        };
        let sock = sock?;

        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .env()
            .init();

        #[get("/ping")]
        fn ping() -> &'static str {
            "pong\n"
        }

        #[get("/check_headers")]
        fn check_headers(req: Request) -> String {
            let headers = req.get_headers();
            let test_header = headers.get("test");
            if let Some(header) = test_header {
                format!("test: {header}")
            } else {
                String::from("No header")
            }
        }

        #[post("/check_post")]
        fn check_post(req: Request) -> String {
            let body = req.get_parsed_body().unwrap();
            format!("hello, {body}")
        }

        #[get("/get_wildcard/:")]
        fn get_wildcard(req: Request) -> String {
            let wildcard = req.get_wildcard().unwrap();
            format!("got: {wildcard}")
        }

        #[post("/post_wildcard/:")]
        fn post_wildcard(req: Request) -> String {
            let wildcard = req.get_wildcard().unwrap();
            format!("got: {wildcard}")
        }

        let routes = Routes::new(vec![
            ping(),
            check_headers(),
            check_post(),
            get_wildcard(),
            post_wildcard(),
        ]);
        let config = Config::new().routes(routes);
        std::thread::spawn(move || {
            HttpListener::new(sock, config).start();
        });
        HTTP_ENABLED.set(true).unwrap();

        Ok(())
    }

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
        assert!(routes
            .clone()
            .get_stream()
            .clone()
            .first()
            .unwrap()
            .wildcard()
            .is_none());
        let request = Request::new(
            b"Hello",
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
                .get_stream()
                .last()
                .unwrap()
                .to_res(request)
                .body
                .unwrap()
        );
    }
    #[test]
    fn respond_to_minimal_request() -> Result<(), Box<dyn std::error::Error>> {
        if HTTP_ENABLED.get().is_none() {
            setup_http_server()?;
        }

        thread::sleep(Duration::from_secs(1));
        let request = minreq::get("http://127.0.0.1:23195/ping").send()?;
        let parsed_resp = request.as_str()?;
        assert_eq!(parsed_resp, "pong\n");

        Ok(())
    }

    #[test]
    fn check_headers() -> Result<(), Box<dyn std::error::Error>> {
        if HTTP_ENABLED.get().is_none() {
            setup_http_server()?;
        }
        thread::sleep(Duration::from_millis(500));
        let req = minreq::get("http://127.0.0.1:23195/check_headers")
            .with_header("test", "yes")
            .send()?;

        assert_eq!(req.as_str()?, "test: yes");

        Ok(())
    }

    #[test]
    fn check_post() -> Result<(), Box<dyn std::error::Error>> {
        if HTTP_ENABLED.get().is_none() {
            setup_http_server()?;
        }
        thread::sleep(Duration::from_millis(100));
        let req = minreq::post("http://127.0.0.1:23195/check_post")
            .with_body("mateo")
            .send()?;

        assert_eq!(req.as_str()?, "hello, mateo");
        Ok(())
    }

    #[test]
    fn check_wildcards() -> Result<(), Box<dyn std::error::Error>> {
        if HTTP_ENABLED.get().is_none() {
            setup_http_server()?;
        }
        thread::sleep(Duration::from_millis(100));
        let get_req = minreq::get("http://127.0.0.1:23195/get_wildcard/tinyhttp").send()?;
        assert_eq!(get_req.as_str()?, "got: tinyhttp");

        let post_req = minreq::post("http://127.0.0.1:23195/post_wildcard/tinyhttp").send()?;
        assert_eq!(post_req.as_str()?, "got: tinyhttp");

        Ok(())
    }
}
