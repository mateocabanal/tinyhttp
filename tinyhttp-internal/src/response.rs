use std::{
    collections::HashMap,
    error::Error,
    io::{Read, Write},
};

#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Debug)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub status_line: String,
    pub body: Option<Vec<u8>>,
    pub mime: Option<String>,
    pub http2: bool,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Response {
    fn from(value: &'a str) -> Self {
        Response::new()
            .body(value.into())
            .mime("text/plain")
            .status_line("HTTP/1.1 200 OK")
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Response::new()
            .body(value.into_bytes())
            .mime("text/plain")
            .status_line("HTTP/1.1 200 OK")
    }
}

impl From<Vec<u8>> for Response {
    fn from(value: Vec<u8>) -> Self {
        Response::new()
            .body(value)
            .mime("application/octet-stream")
            .status_line("HTTP/1.1 200 OK")
    }
}

impl From<()> for Response {
    fn from(_value: ()) -> Self {
        Response::new()
            .body(vec![])
            .mime("text/plain")
            .status_line("HTTP/1.1 403 Forbidden")
    }
}

impl<T: Into<Response>, E: Error + Into<Response>> From<Result<T, E>> for Response {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(body) => body.into(),
            Err(e) => e.into(),
        }
    }
}

impl From<Box<dyn Error>> for Response {
    fn from(value: Box<dyn Error>) -> Self {
        Response::new()
            .body(value.to_string().into_bytes())
            .mime("text/plain")
            .status_line("HTTP/1.1 403 Forbidden")
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            headers: HashMap::new(),
            status_line: String::new(),
            body: None,
            mime: Some(String::from("HTTP/1.1 200 OK")),
            http2: false,
        }
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn status_line<P: Into<String>>(mut self, line: P) -> Self {
        let line_str = line.into();
        self.status_line = line_str.trim().to_string() + "\r\n";
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn mime<P>(mut self, mime: P) -> Self
    where
        P: Into<String>,
    {
        self.mime = Some(mime.into());
        self
    }

    #[cfg(not(feature = "async"))]
    pub(crate) fn send<P: Read + Write>(&self, sock: &mut P) {
        let line_bytes = self.status_line.as_bytes();
        #[cfg(feature = "log")]
        log::trace!("res status line: {:#?}", self.status_line);

        let mut header_bytes: Vec<u8> = self
            .headers
            .iter()
            .flat_map(|(i, j)| [i.as_bytes(), j.as_bytes()].concat())
            .collect();

        header_bytes.extend(b"\r\n");

        #[cfg(all(feature = "log", debug_assertions))]
        {
            log::trace!(
                "HEADER AS STR: {}",
                String::from_utf8(header_bytes.clone()).unwrap()
            );
            log::trace!(
                "STATUS LINE AS STR: {}",
                std::str::from_utf8(line_bytes).unwrap()
            );
        };

        let full_req: &[u8] = &[
            line_bytes,
            header_bytes.as_slice(),
            self.body.as_ref().unwrap(),
        ]
        .concat();

        #[cfg(feature = "log")]
        log::trace!("size of response: {}", full_req.len());

        sock.write_all(full_req).unwrap();
    }

    #[cfg(feature = "async")]
    pub(crate) async fn send<P: AsyncReadExt + AsyncWriteExt + Unpin>(&self, sock: &mut P) {
        let line_bytes = self.status_line.as_bytes();
        #[cfg(feature = "log")]
        log::trace!("res status line: {:#?}", self.status_line);

        let mut header_bytes: Vec<u8> = self
            .headers
            .iter()
            .flat_map(|s| [s.0.as_bytes(), s.1.as_bytes()].concat())
            .collect();

        header_bytes.extend(b"\r\n");

        #[cfg(all(feature = "log", debug_assertions))]
        {
            log::trace!(
                "HEADER AS STR: {}",
                String::from_utf8(header_bytes.clone()).unwrap()
            );
            log::trace!(
                "STATUS LINE AS STR: {}",
                std::str::from_utf8(line_bytes).unwrap()
            );
        };

        let full_req: &[u8] = &[
            line_bytes,
            header_bytes.as_slice(),
            self.body.as_ref().unwrap(),
        ]
        .concat();

        sock.write_all(full_req).await.unwrap();
    }
}
