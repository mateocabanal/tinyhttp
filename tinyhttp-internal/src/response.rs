use std::collections::HashMap;

#[derive(Clone)]
struct Response<'a> {
    headers: Option<HashMap<String, String>>,
    status_line: Option<Vec<String>>,
    body: Option<Vec<u8>>,
    mime: Option<&'a str>,
}

impl<'a> Response<'a> {
    pub(crate) fn new() -> Response<'static> {
        Response {
            headers: None,
            status_line: None,
            body: None,
            mime: None,
        }
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn status_line(mut self, line: Vec<String>) -> Self {
        self.status_line = Some(line);
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn mime(mut self, mime: &'a str) -> Self {
        self.mime = Some(mime);
        self
    }
}
