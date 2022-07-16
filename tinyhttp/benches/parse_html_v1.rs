use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tinyhttp::prelude::*;
use tinyhttp_internal::http::read_to_vec;

use std::{collections::HashMap, path::Path, rc::Rc};

/// Struct containing data on a single request.
///
/// parsed_body which is a Option<String> that can contain the body as a String
///
/// body is used when the body of the request is not a String

type Request = Request;

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

fn parse_request(body: Vec<u8>, config: Config) {
    let buf = body;
    let mut safe_http_index = buf.windows(2).enumerate();
    let status_line_index_opt = safe_http_index
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i);

    let status_line_index = if status_line_index_opt.is_some() {
        status_line_index_opt.unwrap()
    } else {
        #[cfg(feature = "log")]
        log::info!("failed parsing status line!");

        0usize
    };

    let first_header_index = safe_http_index
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i)
        .unwrap();

    #[cfg(feature = "log")]
    log::debug!(
        "STATUS LINE: {:#?}",
        std::str::from_utf8(&buf[..status_line_index])
    );

    #[cfg(feature = "log")]
    log::debug!(
        "FIRST HEADER: {:#?}",
        std::str::from_utf8(&buf[status_line_index + 2..first_header_index])
    );

    let mut headers = Vec::<String>::new();
    let mut headers_index = vec![first_header_index + 2];
    loop {
        let header_index: usize;
        match safe_http_index
            .find(|(_, w)| matches!(*w, b"\r\n"))
            .map(|(i, _)| i + 2)
        {
            Some(s) => header_index = s,
            _ => break,
        }

        #[cfg(feature = "log")]
        log::trace!("header index: {}", header_index);

        let header =
            String::from_utf8(buf[*headers_index.last().unwrap()..header_index - 2].to_vec())
                .unwrap()
                .to_lowercase();
        if header.is_empty() {
            break;
        }
        #[cfg(feature = "log")]
        log::trace!("HEADER: {:?}", headers);

        headers_index.push(header_index);
        headers.push(header);
    }

    let iter_status_line = String::from_utf8(buf[..status_line_index].to_vec()).unwrap();

    //let headers = parse_headers(http.to_string());
    let str_status_line = Vec::from_iter(iter_status_line.split_whitespace().into_iter());
    let status_line: Rc<Vec<String>> =
        Rc::new(str_status_line.iter().map(|i| String::from(*i)).collect());
    #[cfg(feature = "log")]
    log::debug!("{:#?}", status_line);
    let method = &status_line[0];
    let body_index = buf
        .windows(4)
        .enumerate()
        .find(|(_, w)| matches!(*w, b"\r\n\r\n"))
        .map(|(i, _)| i)
        .unwrap();

    let raw_body = &buf[body_index + 4..];
    #[cfg(feature = "log")]
    log::debug!(
        "BODY (TOP): {:#?}",
        std::str::from_utf8(&buf[body_index + 4..]).unwrap()
    );
    let mut res_headers: Vec<String> = Vec::new();
    let request = Request::new(
        raw_body.to_vec(),
        headers.clone(),
        status_line.to_vec(),
        None,
    );

    let (c_status_line, mut body, mime) = match method.as_str() {
        "GET" => match config.get_routes(status_line[1].to_string()) {
            Some(vec) => {
                #[cfg(feature = "log")]
                log::info!("Found path in routes!");

                let line = "HTTP/1.1 200 OK\r\n";

                let request_new = request.clone().set_wildcard(vec.2.clone());

                (line, vec.1(request_new), "text/plain")
            }

            None => match config.get_mount() {
                Some(old_path) => {
                    let path = old_path.to_owned() + &status_line[1];
                    if Path::new(&path).extension().is_none() && config.get_spa() {
                        let body = read_to_vec(&(old_path.to_owned() + "/index.html")).unwrap();
                        let line = "HTTP/1.1 200 OK\r\n";

                        (line, body, "text/html")
                    } else if Path::new(&path.clone()).is_file() {
                        let body = read_to_vec(path.clone()).unwrap();
                        let line = "HTTP/1.1 200 OK\r\n";
                        let mime = mime_guess::from_path(path.clone())
                            .first_raw()
                            .unwrap_or("text/plain");
                        (line, body, mime)
                    } else if Path::new(&path).is_dir() {
                        if Path::new(&(path.to_owned() + "/index.html")).is_file() {
                            let body = read_to_vec(path.to_owned() + "/index.html").unwrap();

                            let lines = "HTTP/1.1 200 OK\r\n";
                            (lines, body, "text/html")
                        } else {
                            (
                                "HTTP/1.1 404 NOT FOUND\r\n",
                                b"<h1>404 Not Found</h1>".to_vec(),
                                "text/html",
                            )
                        }
                    } else if Path::new(&(path.to_owned() + ".html")).is_file() {
                        let body = read_to_vec(path.to_owned() + ".html").unwrap();
                        let lines = "HTTP/1.1 200 OK\r\n";
                        (lines, body, "text/html")
                    } else {
                        (
                            "HTTP/1.1 404 NOT FOUND\r\n",
                            b"<h1>404 Not Found</h1>".to_vec(),
                            "text/html",
                        )
                    }
                }

                None => (
                    "HTTP/1.1 404 NOT FOUND\r\n",
                    b"<h1>404 Not Found</h1>".to_vec(),
                    "text/html",
                ),
            },
        },
        "POST" => match config.post_routes() {
            Some(vec) => {
                #[cfg(feature = "log")]
                log::info!("POST");

                let mut res = ("", vec![], "");
                let stat_line = status_line.clone();
                for c in vec.iter() {
                    if c.0 == stat_line[1] {
                        let line = "HTTP/1.1 200 OK\r\n";
                        res = (line, c.1(request.clone()), "text/plain");
                    } else if stat_line[1].contains(&c.0) && c.2.is_some() {
                        let line = "HTTP/1.1 200 OK\r\n";
                        let split = stat_line[1].split(&c.0).last().unwrap();
                        let request_new = request.clone().set_wildcard(Some(split.to_string()));
                        res = (line, c.1(request_new), "text/plain");
                    }
                }
                res
            }

            None => (
                "HTTP/1.1 404 NOT FOUND\r\n",
                b"<h1>404 Not Found</h1>".to_vec(),
                "text/html",
            ),
        },

        _ => (
            "HTTP/1.1 404 NOT FOUND\r\n",
            b"<h1>404 Not Found</h1>".to_vec(),
            "text/html",
        ),
    };

    let inferred_mime = match infer::get(&body) {
        Some(mime) => mime.mime_type(),
        None => mime,
    };

    res_headers.push(c_status_line.to_string());
    match config.get_headers() {
        Some(vec) => {
            for i in vec {
                res_headers.push(format!("{}\r\n", i));
            }
        }
        None => (),
    }
    res_headers.push(format!("Content-Type: {}\r\n", inferred_mime));

    res_headers.push("X-:)-->: HEHEHE\r\n".to_string());

    let req_headers = request.clone().get_headers();

    let comp = if req_headers.contains_key("accept-encoding") {
        let tmp_str: String = req_headers.get("accept-encoding").unwrap().to_owned();
        let res = tmp_str.split(",").map(|s| s.trim().to_string()).collect();

        #[cfg(feature = "log")]
        log::debug!("{:#?}", &res);

        res
    } else {
        Vec::new()
    };

    #[cfg(feature = "sys")]
    if req_headers.contains_key("accept-encoding") {
        if config.get_gzip() && comp.contains(&"gzip".to_string()) {
            #[cfg(feature = "log")]
            log::info!("GZIP ENABLED!");
            let start: Instant = std::time::Instant::now();

            let mut writer = GzEncoder::new(Vec::new(), Compression::default());
            writer.write_all(&body).unwrap();
            body = writer.finish().unwrap();
            res_headers.push("Content-Encoding: gzip\r\n".to_string());
            #[cfg(feature = "log")]
            log::info!("COMPRESS TOOK {} SECS", start.elapsed().as_secs());
        } else if config.get_br() && comp.contains(&"br".to_string()) {
            #[cfg(feature = "log")]
            log::info!("BROTLI ENABLED!");
            let br_body = body.clone();
            let start = std::time::Instant::now();
            let mut compressor = brotli2::read::BrotliEncoder::new(&*br_body, 9);
            compressor.read_to_end(&mut body).unwrap();
            #[cfg(feature = "log")]
            log::info!("{:?}", body);
            res_headers.push("Content-Encoding: br\r\n".to_string());
            #[cfg(feature = "log")]
            log::info!("COMPRESS TOOK {} SECS", start.elapsed().as_secs());
        }
    }

    let mut headers: String = String::new();
    for i in res_headers {
        headers += &(i.as_str().to_owned());
    }
    headers += "\r\n";

    #[cfg(feature = "log")]
    log::debug!(
        "RESPONSE BODY: {:#?},\n RESPONSE HEADERS: {:#?}\n",
        body,
        headers
    );

    let res = [headers.as_bytes(), &body[..]].concat();
}
