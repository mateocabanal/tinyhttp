use std::{collections::HashMap, fmt::Display, mem};

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
/// parsed_body which is a Option<String> that can contain the body as a String
///
/// body is used when the body of the request is not a String
#[derive(Clone, Debug, Default)]
pub struct Request {
    raw_headers: HashMap<String, String>,
    status_line: Vec<String>,
    body: Vec<u8>,
    wildcard: Option<String>,
    is_http2: bool,
}

#[derive(Clone, Debug)]
pub enum BodyType {
    ASCII(String),
    Bytes(Vec<u8>),
}

impl Request {
    pub fn new(
        body: Vec<u8>,
        raw_headers: HashMap<String, String>,
        status_line: Vec<String>,
        wildcard: Option<String>,
    ) -> Request {
        Request {
            body,
            raw_headers,
            status_line,
            wildcard,
            is_http2: false,
        }
    }

    pub(crate) fn set_wildcard(&mut self, w: Option<String>) -> &Self {
        self.wildcard = w;
        self
    }

    /// Get request body as bytes
    pub fn get_raw_body(&self) -> &[u8] {
        &self.body
    }

    /// Get request body as a string
    pub fn get_parsed_body(&self) -> Option<&str> {
        if let Ok(s) = std::str::from_utf8(&self.body) {
            Some(s)
        } else {
            None
        }
    }

    /// Get request headers in a HashMap
    pub fn get_headers(&self) -> &HashMap<String, String> {
        #[cfg(feature = "log")]
        log::trace!("Headers: {:#?}", self.raw_headers);

        &self.raw_headers
    }

    /// Get status line of request
    pub fn get_status_line(&self) -> &[String] {
        &self.status_line
    }

    pub fn get_wildcard(&self) -> Option<&String> {
        self.wildcard.as_ref()
    }

    pub fn get_http2(&self) -> bool {
        self.is_http2
    }

    pub fn set_http2(mut self, w: bool) -> Self {
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

//impl<'a> From<&'a mut Request> for Wildcard<&'a [u8]> {
//    fn from(value: &'a mut Request) -> Self {
//        Wildcard {
//            wildcard: value.wildcard.as_ref().unwrap().as_bytes(),
//        }
//    }
//}

impl<'a> From<&'a mut Request> for &'a HashMap<String, String> {
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
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("failed to parse status line")]
    StatusLineErr,
    #[error("failed to parse headers")]
    HeadersErr,
}
