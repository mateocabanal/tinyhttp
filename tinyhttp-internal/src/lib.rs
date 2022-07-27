//#![warn(missing_docs)]

pub mod codegen;
pub mod config;
pub mod request;
pub mod response;
pub mod thread_pool;

#[cfg(not(feature = "async"))]
pub mod http;

#[cfg(feature = "async")]
pub mod async_http;

<<<<<<< HEAD
pub mod http2;
=======

#[cfg(test)]
mod tests {
    #[test]
    fn build_request() {
        use crate::request::Request;
        let request = Request::new(b"Hello, World!".to_vec(), vec!["Content-Type: text/plain".to_string()], vec!["GET".to_string(), "/test".to_string(), "HTTP/1.1".to_string()], None);
        assert_eq!(request.get_parsed_body().unwrap(), "Hello, World!".to_string())
    }
}
>>>>>>> 8391d6a (add unit tests)
