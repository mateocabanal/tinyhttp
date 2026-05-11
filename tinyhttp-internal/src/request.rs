use std::{fmt::Display, mem};

use thiserror::Error;

use crate::{config::Method, headers::HeaderMap};

#[derive(Clone, Debug, Default)]
pub struct Wildcard<T: Display> {
    wildcard: T,
}

impl<T: Display> Display for Wildcard<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_wildcard())
    }
}

impl<T: Display> Wildcard<T> {
    pub fn get_wildcard(&self) -> &T {
        &self.wildcard
    }
}

/// Struct containing data on a single request.
///
/// `body` stores the raw request body. `get_parsed_body` returns a borrowed
/// UTF-8 view when the body is valid UTF-8.
#[derive(Clone, Debug)]
pub struct Request {
    raw_headers: HeaderMap,
    status_line: Vec<String>,
    method: Method,
    path: String,
    version: String,
    body: Vec<u8>,
    wildcard: Option<String>,
    is_http2: bool,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            raw_headers: HeaderMap::new(),
            status_line: vec!["GET".to_string(), "/".to_string(), "HTTP/1.1".to_string()],
            method: Method::GET,
            path: "/".to_string(),
            version: "HTTP/1.1".to_string(),
            body: Vec::new(),
            wildcard: None,
            is_http2: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum BodyType {
    ASCII(String),
    Bytes(Vec<u8>),
}

impl Request {
    pub fn new(
        body: Vec<u8>,
        raw_headers: HeaderMap,
        status_line: Vec<String>,
        wildcard: Option<String>,
    ) -> Request {
        let method = status_line
            .first()
            .map(|method| Method::from_str(method))
            .unwrap_or(Method::GET);
        let path = status_line
            .get(1)
            .cloned()
            .unwrap_or_else(|| "/".to_string());
        let version = status_line
            .get(2)
            .cloned()
            .unwrap_or_else(|| "HTTP/1.1".to_string());

        Request {
            body,
            raw_headers,
            status_line,
            method,
            path,
            version,
            wildcard,
            is_http2: false,
        }
    }

    pub(crate) fn new_parts(
        body: Vec<u8>,
        raw_headers: HeaderMap,
        method: Method,
        path: String,
        version: String,
        wildcard: Option<String>,
    ) -> Request {
        let status_line = vec![method.as_str().to_string(), path.clone(), version.clone()];

        Request {
            body,
            raw_headers,
            status_line,
            method,
            path,
            version,
            wildcard,
            is_http2: false,
        }
    }

    pub(crate) fn set_wildcard(&mut self, w: Option<String>) -> &Self {
        self.wildcard = w;
        self
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// Get request body as bytes.
    pub fn get_raw_body(&self) -> &[u8] {
        &self.body
    }

    /// Get request body as a string.
    pub fn get_parsed_body(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }

    /// Get request headers.
    pub fn get_headers(&self) -> &HeaderMap {
        #[cfg(feature = "log")]
        log::trace!("Headers: {:#?}", self.raw_headers);

        &self.raw_headers
    }

    /// Get status line of request.
    ///
    /// This legacy accessor remains for compatibility. New internal code should
    /// prefer `method()`, `path()`, and `version()` to avoid vector indexing.
    pub fn get_status_line(&self) -> &[String] {
        &self.status_line
    }

    pub fn get_wildcard(&self) -> Option<&String> {
        self.wildcard.as_ref()
    }

    pub fn get_http2(&self) -> bool {
        self.is_http2
    }

    #[allow(dead_code)]
    pub(crate) fn set_http2(mut self, w: bool) -> Self {
        self.is_http2 = w;
        self
    }
}

impl<'a> From<&'a mut Request> for Wildcard<&'a str> {
    fn from(value: &'a mut Request) -> Self {
        Wildcard {
            wildcard: value.wildcard.as_ref().unwrap(),
        }
    }
}

// TODO: Add docs here
impl<'a> From<&'a mut Request> for &'a HeaderMap {
    fn from(value: &'a mut Request) -> Self {
        value.get_headers()
    }
}

impl<'a> From<&'a mut Request> for &'a [u8] {
    fn from(value: &'a mut Request) -> Self {
        value.get_raw_body()
    }
}

impl<'a> From<&'a mut Request> for Option<&'a str> {
    fn from(value: &'a mut Request) -> Self {
        value.get_parsed_body()
    }
}

impl From<&mut Request> for Request {
    fn from(value: &mut Request) -> Self {
        mem::take(value)
    }
}

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("failed to parse status line")]
    StatusLineErr,
    #[error("failed to parse headers")]
    HeadersErr,
}
