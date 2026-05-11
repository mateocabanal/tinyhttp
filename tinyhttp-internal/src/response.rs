use std::{
    borrow::Cow,
    error::Error,
    io::{self, IoSlice, Read, Write},
    sync::Arc,
};

use smallvec::SmallVec;

#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub type ResponseHeader = (Cow<'static, str>, Cow<'static, str>);

#[derive(Clone, Debug)]
pub struct Response {
    pub headers: SmallVec<[ResponseHeader; 8]>,
    pub status_line: Cow<'static, str>,
    pub body: Option<Vec<u8>>,
    pub mime: Option<Cow<'static, str>>,
    pub http2: bool,
    pub(crate) manual_override: bool,
    static_body: Option<&'static [u8]>,
    prebuilt: Option<Arc<[u8]>>,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Response {
    fn from(value: &'a str) -> Self {
        Response::new()
            .body(value.as_bytes().to_vec())
            .mime("text/plain")
            .status_line("HTTP/1.1 200 OK\r\n")
    }
}

impl From<&'static [u8]> for Response {
    fn from(value: &'static [u8]) -> Self {
        Response::new()
            .body_static(value)
            .mime("application/octet-stream")
            .status_line("HTTP/1.1 200 OK\r\n")
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Response::new()
            .body(value.into_bytes())
            .mime("text/plain")
            .status_line("HTTP/1.1 200 OK\r\n")
    }
}

impl From<Vec<u8>> for Response {
    fn from(value: Vec<u8>) -> Self {
        Response::new()
            .body(value)
            .mime("application/octet-stream")
            .status_line("HTTP/1.1 200 OK\r\n")
    }
}

impl From<()> for Response {
    fn from(_value: ()) -> Self {
        Response::new()
            .body(vec![])
            .mime("text/plain")
            .status_line("HTTP/1.1 404 Not Found\r\n")
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
            .status_line("HTTP/1.1 403 Forbidden\r\n")
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            headers: SmallVec::new(),
            mime: None,
            body: None,
            status_line: Cow::Borrowed("HTTP/1.1 200 OK\r\n"),
            http2: false,
            manual_override: false,
            static_body: None,
            prebuilt: None,
        }
    }

    pub fn empty() -> Response {
        Response {
            headers: SmallVec::new(),
            status_line: Cow::Borrowed(""),
            body: None,
            mime: None,
            manual_override: true,
            http2: false,
            static_body: None,
            prebuilt: None,
        }
    }

    pub fn prebuilt(bytes: Arc<[u8]>) -> Response {
        Response {
            headers: SmallVec::new(),
            status_line: Cow::Borrowed(""),
            body: None,
            mime: None,
            manual_override: false,
            http2: false,
            static_body: None,
            prebuilt: Some(bytes),
        }
    }

    pub fn headers(mut self, headers: std::collections::HashMap<String, String>) -> Self {
        self.headers = headers
            .into_iter()
            .map(|(key, value)| (Cow::Owned(key), Cow::Owned(value)))
            .collect();
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        self.insert_header(key, value);
        self
    }

    pub fn status_line<P: Into<Cow<'static, str>>>(mut self, line: P) -> Self {
        let line = line.into();
        let s = line.as_ref();

        if let Some(without_crlf) = s.strip_suffix("\r\n") {
            if without_crlf.trim() == without_crlf {
                self.status_line = line;
                return self;
            }
        }

        let mut line_str = s.trim().to_string();
        line_str.push_str("\r\n");
        self.status_line = Cow::Owned(line_str);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.static_body = None;
        self.body = Some(body);
        self
    }

    pub fn body_static(mut self, body: &'static [u8]) -> Self {
        self.body = None;
        self.static_body = Some(body);
        self
    }

    pub fn mime<P>(mut self, mime: P) -> Self
    where
        P: Into<Cow<'static, str>>,
    {
        self.mime = Some(mime.into());
        self
    }

    pub fn body_bytes(&self) -> Option<&[u8]> {
        self.static_body.or(self.body.as_deref())
    }

    pub fn body_len(&self) -> usize {
        self.body_bytes().map(|body| body.len()).unwrap_or(0)
    }

    pub(crate) fn is_prebuilt(&self) -> bool {
        self.prebuilt.is_some()
    }

    pub(crate) fn insert_header<K, V>(&mut self, key: K, value: V)
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let key = key.into();
        let value = value.into();

        if let Some((_, old_value)) = self
            .headers
            .iter_mut()
            .find(|(old_key, _)| old_key.eq_ignore_ascii_case(key.as_ref()))
        {
            *old_value = value;
            return;
        }

        self.headers.push((key, value));
    }

    pub(crate) fn extend_headers<I, K, V>(&mut self, headers: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        for (key, value) in headers {
            self.insert_header(key, value);
        }
    }

    pub(crate) fn has_header(&self, key: &str) -> bool {
        self.headers
            .iter()
            .any(|(header, _)| header.eq_ignore_ascii_case(key))
    }

    pub(crate) fn add_tinyhttp_header(&mut self) {
        self.insert_header("tinyhttp", env!("CARGO_PKG_VERSION"));
    }

    pub(crate) fn add_content_type_if_missing(&mut self) {
        if self.has_header("Content-Type") {
            return;
        }

        if let Some(mime) = self.mime.clone() {
            self.insert_header("Content-Type", mime);
        } else if self.body_bytes().is_some() {
            self.insert_header("Content-Type", "text/plain");
        }
    }

    pub(crate) fn replace_body(&mut self, body: Vec<u8>) {
        self.static_body = None;
        self.body = Some(body);
    }

    pub(crate) fn render_cached_bytes(mut self) -> Vec<u8> {
        self.add_content_type_if_missing();
        self.add_tinyhttp_header();

        let mut out = Vec::with_capacity(
            self.status_line.len() + self.header_bytes_len() + self.body_len() + 32,
        );
        out.extend_from_slice(self.status_line.as_bytes());
        self.write_headers_to_vec(&mut out);
        out.extend_from_slice(b"\r\n");

        if let Some(body) = self.body_bytes() {
            out.extend_from_slice(body);
        }

        out
    }

    fn header_bytes_len(&self) -> usize {
        self.headers
            .iter()
            .map(|(key, value)| key.len() + 2 + value.len() + 2)
            .sum()
    }

    fn write_headers_to_vec(&self, out: &mut Vec<u8>) {
        for (key, value) in &self.headers {
            out.extend_from_slice(key.as_bytes());
            out.extend_from_slice(b": ");
            out.extend_from_slice(value.as_bytes());
            out.extend_from_slice(b"\r\n");
        }

        self.write_content_length_to_vec(out);
    }

    fn write_headers_to_smallvec(&self, out: &mut SmallVec<[u8; 512]>) {
        for (key, value) in &self.headers {
            out.extend_from_slice(key.as_bytes());
            out.extend_from_slice(b": ");
            out.extend_from_slice(value.as_bytes());
            out.extend_from_slice(b"\r\n");
        }

        if !self.has_header("Content-Length") {
            let mut len_buf = itoa::Buffer::new();
            let len = len_buf.format(self.body_len());
            out.extend_from_slice(b"Content-Length: ");
            out.extend_from_slice(len.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
    }

    fn write_content_length_to_vec(&self, out: &mut Vec<u8>) {
        if !self.has_header("Content-Length") {
            let mut len_buf = itoa::Buffer::new();
            let len = len_buf.format(self.body_len());
            out.extend_from_slice(b"Content-Length: ");
            out.extend_from_slice(len.as_bytes());
            out.extend_from_slice(b"\r\n");
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn send<P: Read + Write>(self, sock: &mut P) {
        if let Some(prebuilt) = self.prebuilt {
            sock.write_all(&prebuilt).unwrap();
            return;
        }

        #[cfg(feature = "log")]
        log::trace!("res status line: {:#?}", self.status_line);

        let mut header_bytes: SmallVec<[u8; 512]> = SmallVec::new();
        header_bytes.reserve(self.header_bytes_len() + 32);
        self.write_headers_to_smallvec(&mut header_bytes);
        header_bytes.extend_from_slice(b"\r\n");

        let body = self.body_bytes().unwrap_or(&[]);

        #[cfg(feature = "log")]
        log::trace!(
            "size of response: {}",
            self.status_line.len() + header_bytes.len() + body.len()
        );

        write_all_vectored(
            sock,
            [self.status_line.as_bytes(), header_bytes.as_slice(), body],
        )
        .unwrap();
    }

    #[cfg(feature = "async")]
    pub(crate) async fn send<P: AsyncReadExt + AsyncWriteExt + Unpin>(&self, sock: &mut P) {
        if let Some(prebuilt) = &self.prebuilt {
            sock.write_all(prebuilt).await.unwrap();
            return;
        }

        let bytes = self.clone().render_cached_bytes();
        sock.write_all(&bytes).await.unwrap();
    }
}

#[cfg(not(feature = "async"))]
fn write_all_vectored<W: Write>(writer: &mut W, parts: [&[u8]; 3]) -> io::Result<()> {
    let slices = [
        IoSlice::new(parts[0]),
        IoSlice::new(parts[1]),
        IoSlice::new(parts[2]),
    ];

    let written = writer.write_vectored(&slices)?;

    if written == 0 && parts.iter().any(|part| !part.is_empty()) {
        return Err(io::Error::from(io::ErrorKind::WriteZero));
    }

    let total: usize = parts.iter().map(|part| part.len()).sum();
    if written >= total {
        return Ok(());
    }

    let mut remaining_skip = written;
    for part in parts {
        if remaining_skip >= part.len() {
            remaining_skip -= part.len();
            continue;
        }

        writer.write_all(&part[remaining_skip..])?;
        remaining_skip = 0;
    }

    Ok(())
}
