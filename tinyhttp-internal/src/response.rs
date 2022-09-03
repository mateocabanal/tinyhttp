use std::{
    collections::HashMap,
    io::{Read, Write},
};

#[derive(Clone)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub status_line: String,
    pub body: Option<Vec<u8>>,
    pub mime: Option<String>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            headers: HashMap::new(),
            status_line: String::new(),
            body: None,
            mime: None,
        }
    }

    #[allow(dead_code)]
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn status_line<P: Into<String>>(mut self, line: P) -> Self {
        self.status_line = line.into();
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

    pub(crate) fn send<P: Read + Write>(&self, sock: &mut P) {
        let line_bytes = self.status_line.as_bytes().to_vec();
        #[cfg(feature = "log")]
        log::debug!("res status line: {:#?}", self.status_line);

        let mut header_bytes = Vec::from_iter(
            self.headers
                .iter()
                .flat_map(|s| [s.0.as_bytes(), s.1.as_bytes()]),
        );
        let mut full_req = Vec::new();
        for i in line_bytes {
            full_req.push(i);
        }
        header_bytes.push(b"\r\n");
        for i in header_bytes {
            for j in i {
                full_req.push(*j);
            }
        }

        #[cfg(feature = "log")]
        log::debug!(
            "RESPONSE HEADERS (AFTER PARSE): {}",
            String::from_utf8(full_req.clone()).unwrap()
        );

        if let Some(i) = self.body.as_ref() {
            for j in i {
                full_req.push(*j);
            }
        }

        sock.write_all(&full_req).unwrap();
    }
}
