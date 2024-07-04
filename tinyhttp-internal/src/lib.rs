//#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod codegen;
pub mod config;
pub mod request;
pub mod response;

#[cfg(not(feature = "async"))]
pub mod http;

#[cfg(feature = "async")]
pub mod async_http;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn build_request() {
        use crate::request::Request;
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/plain".to_string());

        let request = Request::new(
            b"Hello, World!".to_vec(),
            headers,
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
