//#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod codegen;
pub mod config;
pub mod request;
pub mod response;
pub mod thread_pool;

pub mod http;

#[cfg(test)]
mod tests {
    #[test]
    fn build_request() {
        use crate::request::Request;
        let request = Request::new(
            b"Hello, World!".to_vec(),
            vec!["Content-Type: text/plain".to_string()],
            vec![
                "GET".to_string(),
                "/test".to_string(),
                "HTTP/1.1".to_string(),
            ],
            None,
        );
        assert_eq!(
            *request.get_parsed_body().unwrap(),
            "Hello, World!".to_string()
        )
    }
    #[test]
    fn build_response() {
        use crate::response::Response;

        let response = Response::new()
            .body(b"1 2 3 test test...".to_vec())
            .status_line("HTTP/1.1 200 OK");

        assert_eq!(
            String::from_utf8(response.body.unwrap()).unwrap(),
            String::from("1 2 3 test test...")
        );
    }
}
