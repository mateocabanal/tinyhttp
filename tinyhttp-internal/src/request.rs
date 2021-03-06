use std::collections::HashMap;

/// Struct containing data on a single request.
///
/// parsed_body which is a Option<String> that can contain the body as a String
///
/// body is used when the body of the request is not a String

#[derive(Clone)]
pub struct Request {
    parsed_body: Option<String>,
    headers: HashMap<String, String>,
    status_line: Vec<String>,
    body: Vec<u8>,
    wildcard: Option<String>,
}

#[derive(Clone, Debug)]
pub enum BodyType {
    ASCII(String),
    Bytes(Vec<u8>),
}

impl Request {
    pub(crate) fn new(
        raw_body: Vec<u8>,
        raw_headers: Vec<String>,
        status_line: Vec<String>,
        wildcard: Option<String>,
    ) -> Request {
        let raw_body_clone = raw_body.clone();
        let ascii_body = match std::str::from_utf8(&raw_body_clone) {
            Ok(s) => Some(s),
            Err(_) => {
                #[cfg(feature = "log")]
                log::info!("Not an ASCII body");
                None
            }
        };

        let mut headers: HashMap<String, String> = HashMap::new();
        #[cfg(feature = "log")]
        log::trace!("Headers: {:#?}", raw_headers);
        for i in raw_headers.iter() {
            let mut iter = i.split(": ");
            let key = iter.next().unwrap();
            let value = iter.next().unwrap();

            /*            match value {
                            Some(v) => println!("{}", v),
                            None => {
                                break;
                            }
                        };
            */
            headers.insert(key.to_string(), value.to_string());
        }

        #[cfg(feature = "log")]
        log::info!("Request headers: {:?}", headers);

        if ascii_body.is_none() {
            Request {
                parsed_body: None,
                body: raw_body,
                headers,
                status_line,
                wildcard,
            }
        } else {
            Request {
                body: raw_body,
                parsed_body: Some(ascii_body.unwrap().to_string()),
                headers,
                status_line,
                wildcard,
            }
        }
    }

    pub(crate) fn set_wildcard(mut self, w: Option<String>) -> Self {
        self.wildcard = w;
        self
    }

    pub fn get_raw_body(&self) -> Vec<u8> {
        self.body.clone()
    }

    pub fn get_parsed_body(&self) -> Option<String> {
        self.parsed_body.clone()
    }

    pub fn get_headers(&self) -> HashMap<String, String> {
        self.headers.clone()
    }

    pub fn get_status_line(&self) -> Vec<String> {
        self.status_line.clone()
    }

    pub fn get_wildcard(&self) -> Option<String> {
        self.wildcard.clone()
    }
}
