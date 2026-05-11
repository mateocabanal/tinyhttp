use std::{
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
    path::Path,
    sync::Arc,
};

use memchr::{memchr, memmem};

use crate::{
    config::{Config, HttpListener, Method},
    headers::HeaderMap,
    request::{Request, RequestError},
    response::Response,
};

#[cfg(feature = "sys")]
use flate2::{write::GzEncoder, Compression};

const MAX_HEADER_BYTES: usize = 16 * 1024;

pub fn start_http(http: HttpListener, config: Config) {
    #[cfg(feature = "log")]
    log::info!(
        "Listening on port {}",
        http.socket.local_addr().unwrap().port()
    );

    let arc_config = Arc::new(config);
    for stream in http.get_stream() {
        let mut conn = stream.unwrap();

        let config = arc_config.clone();
        if http.use_pool {
            http.pool.execute(move || {
                #[cfg(feature = "log")]
                log::trace!("parse_request() called");

                parse_request(&mut conn, config);
            });
        } else {
            #[cfg(feature = "log")]
            log::trace!("parse_request() called");

            parse_request(&mut conn, config);
        }
    }
}

#[derive(Clone, Debug)]
struct ParsedRequestLine {
    method: Method,
    path: String,
    version: String,
}

fn parse_request_line(line: &str) -> Result<ParsedRequestLine, RequestError> {
    let line = line.trim_end_matches(['\r', '\n']);
    let mut parts = line.split_whitespace();

    let method = parts.next().ok_or(RequestError::StatusLineErr)?;
    let path = parts.next().ok_or(RequestError::StatusLineErr)?;
    let version = parts.next().ok_or(RequestError::StatusLineErr)?;

    Ok(ParsedRequestLine {
        method: Method::from_str(method),
        path: path.to_string(),
        version: version.to_string(),
    })
}

fn build_and_parse_req<P: Read>(conn: &mut P) -> Result<Request, RequestError> {
    let mut buf_reader = BufReader::with_capacity(8192, conn);
    let mut status_line_str = String::new();
    buf_reader
        .read_line(&mut status_line_str)
        .map_err(|_| RequestError::StatusLineErr)?;

    let request_line = parse_request_line(&status_line_str)?;
    build_and_parse_req_from_reader(&mut buf_reader, request_line)
}

fn build_and_parse_req_from_reader<P: Read>(
    buf_reader: &mut BufReader<P>,
    request_line: ParsedRequestLine,
) -> Result<Request, RequestError> {
    #[cfg(feature = "log")]
    log::trace!(
        "STATUS LINE: {} {} {}",
        request_line.method.as_str(),
        request_line.path,
        request_line.version
    );

    let mut headers_buf = Vec::with_capacity(1024);

    loop {
        let base = headers_buf.len();
        let available = buf_reader
            .fill_buf()
            .map_err(|_| RequestError::HeadersErr)?;

        if available.is_empty() {
            return Err(RequestError::HeadersErr);
        }

        headers_buf.extend_from_slice(available);

        if let Some(header_end) = memmem::find(&headers_buf, b"\r\n\r\n") {
            let consumed_from_available = (header_end + 4)
                .saturating_sub(base)
                .min(available.len());

            buf_reader.consume(consumed_from_available);
            headers_buf.truncate(header_end + 2);
            break;
        }

        if headers_buf.len() > MAX_HEADER_BYTES {
            return Err(RequestError::HeadersErr);
        }

        let len = available.len();
        buf_reader.consume(len);
    }

    let mut headers = HeaderMap::with_capacity(16);

    for line in headers_buf.split(|byte| *byte == b'\n') {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        if line.is_empty() {
            break;
        }

        let colon_idx = memchr(b':', line).ok_or(RequestError::HeadersErr)?;
        let key = std::str::from_utf8(&line[..colon_idx]).map_err(|_| RequestError::HeadersErr)?;
        let value =
            std::str::from_utf8(trim_ascii(&line[colon_idx + 1..])).map_err(|_| RequestError::HeadersErr)?;

        headers.set(key, value);
    }

    let body_len = headers
        .get("Content-Length")
        .map(|str| str.parse::<usize>().unwrap())
        .unwrap_or(0usize);

    let mut raw_body = vec![0; body_len];
    buf_reader
        .read_exact(&mut raw_body)
        .map_err(|_| RequestError::HeadersErr)?;

    Ok(Request::new_parts(
        raw_body,
        headers,
        request_line.method,
        request_line.path,
        request_line.version,
        None,
    ))
}

fn trim_ascii(mut bytes: &[u8]) -> &[u8] {
    while let Some((first, rest)) = bytes.split_first() {
        if !first.is_ascii_whitespace() {
            break;
        }
        bytes = rest;
    }

    while let Some((last, rest)) = bytes.split_last() {
        if !last.is_ascii_whitespace() {
            break;
        }
        bytes = rest;
    }

    bytes
}

fn build_res(mut req: Request, config: &Config, sock: &mut TcpStream) -> Response {
    #[cfg(feature = "log")]
    log::trace!("build_res -> req_path: {}", req.path());

    match req.method() {
        Method::GET => match config.get_routes(req.path()) {
            Some(route) => {
                #[cfg(feature = "log")]
                log::trace!("Found path in routes!");

                let wildcard = route.wildcard().and_then(|_| {
                    req.path()
                        .strip_prefix(route.get_path())
                        .and_then(|suffix| suffix.strip_prefix('/'))
                        .map(ToOwned::to_owned)
                });

                if wildcard.is_some() {
                    req.set_wildcard(wildcard);
                }

                route.to_res(req, sock)
            }

            None => match config.get_mount() {
                Some(old_path) => {
                    let path = old_path.to_owned() + req.path();
                    if Path::new(&path).extension().is_none() && config.get_spa() {
                        let body = read_to_vec(old_path.to_owned() + "/index.html").unwrap();
                        let line = "HTTP/1.1 200 OK\r\n";

                        Response::new()
                            .status_line(line)
                            .body(body)
                            .mime("text/html")
                    } else if Path::new(&path).is_file() {
                        let body = read_to_vec(&path).unwrap();
                        let line = "HTTP/1.1 200 OK\r\n";
                        let mime = mime_guess::from_path(&path)
                            .first_raw()
                            .unwrap_or("text/plain");
                        Response::new().status_line(line).body(body).mime(mime)
                    } else if Path::new(&path).is_dir() {
                        if Path::new(&(path.to_owned() + "/index.html")).is_file() {
                            let body = read_to_vec(path + "/index.html").unwrap();

                            let line = "HTTP/1.1 200 OK\r\n";
                            Response::new()
                                .status_line(line)
                                .body(body)
                                .mime("text/html")
                        } else {
                            Response::new()
                                .status_line("HTTP/1.1 404 NOT FOUND\r\n")
                                .body(b"<h1>404 Not Found</h1>".to_vec())
                                .mime("text/html")
                        }
                    } else if Path::new(&(path.to_owned() + ".html")).is_file() {
                        let body = read_to_vec(path + ".html").unwrap();
                        let line = "HTTP/1.1 200 OK\r\n";
                        Response::new()
                            .status_line(line)
                            .body(body)
                            .mime("text/html")
                    } else {
                        Response::new()
                            .status_line("HTTP/1.1 404 NOT FOUND\r\n")
                            .body(b"<h1>404 Not Found</h1>".to_vec())
                            .mime("text/html")
                    }
                }

                None => Response::new()
                    .status_line("HTTP/1.1 404 NOT FOUND\r\n")
                    .body(b"<h1>404 Not Found</h1>".to_vec())
                    .mime("text/html"),
            },
        },
        Method::POST => match config.post_routes(req.path()) {
            Some(route) => {
                #[cfg(feature = "log")]
                log::debug!("POST");

                let wildcard = route.wildcard().and_then(|_| {
                    req.path()
                        .strip_prefix(route.get_path())
                        .and_then(|suffix| suffix.strip_prefix('/'))
                        .map(ToOwned::to_owned)
                });

                if wildcard.is_some() {
                    req.set_wildcard(wildcard);
                }

                route.to_res(req, sock)
            }

            None => Response::new()
                .status_line("HTTP/1.1 404 NOT FOUND\r\n")
                .body(b"<h1>404 Not Found</h1>".to_vec())
                .mime("text/html"),
        },
    }
}

pub fn parse_request(conn: &mut TcpStream, config: Arc<Config>) {
    let mut buf_reader = BufReader::with_capacity(8192, conn);
    let mut status_line_str = String::new();

    if buf_reader.read_line(&mut status_line_str).is_err() {
        Response::new()
            .mime("text/plain")
            .body(b"failed to parse status line".to_vec())
            .send(buf_reader.get_mut());
        return;
    }

    let request_line = match parse_request_line(&status_line_str) {
        Ok(line) => line,
        Err(_) => {
            Response::new()
                .mime("text/plain")
                .body(b"failed to parse status line".to_vec())
                .send(buf_reader.get_mut());
            return;
        }
    };

    if let Some(route) = config.route_for(request_line.method, &request_line.path) {
        if !route.needs_request() && !config.get_gzip() {
            if config.can_use_prebuilt_routes() {
                if let Some(cached) = route.cached_response() {
                    buf_reader.get_mut().write_all(cached).unwrap();
                    return;
                }
            }

            let request = Request::new_parts(
                Vec::new(),
                HeaderMap::new(),
                request_line.method,
                request_line.path.clone(),
                request_line.version.clone(),
                None,
            );
            let response = route.to_res(request, buf_reader.get_mut());

            if response.manual_override {
                buf_reader
                    .get_mut()
                    .shutdown(std::net::Shutdown::Both)
                    .unwrap();
                return;
            }

            finish_response(response, &config, false, buf_reader.get_mut());
            return;
        }
    }

    let request = build_and_parse_req_from_reader(&mut buf_reader, request_line);

    let request = match request {
        Ok(request) => request,
        Err(e) => {
            let specific_err = match e {
                RequestError::StatusLineErr => b"failed to parse status line".to_vec(),
                RequestError::HeadersErr => b"failed to parse headers".to_vec(),
            };
            Response::new()
                .mime("text/plain")
                .body(specific_err)
                .send(buf_reader.get_mut());

            return;
        }
    };

    let compress = config.get_gzip()
        && request
            .get_headers()
            .get("Accept-Encoding")
            .map(|tmp_str| {
                tmp_str
                    .split(',')
                    .any(|encoding| encoding.trim().eq_ignore_ascii_case("gzip"))
            })
            .unwrap_or(false);

    let response = build_res(request, &config, buf_reader.get_mut());
    if response.manual_override {
        buf_reader
            .get_mut()
            .shutdown(std::net::Shutdown::Both)
            .unwrap();
        return;
    }

    finish_response(response, &config, compress, buf_reader.get_mut());
}

fn finish_response(mut response: Response, config: &Config, compress: bool, conn: &mut TcpStream) {
    if response.is_prebuilt() {
        response.send(conn);
        return;
    }

    response.add_content_type_if_missing();

    if let Some(config_headers) = config.get_headers() {
        response.extend_headers(
            config_headers
                .iter()
                .map(|(i, j)| (i.to_owned(), j.to_owned())),
        );
    }

    response.add_tinyhttp_header();

    #[cfg(feature = "sys")]
    {
        if compress && response.body_len() >= 512 {
            use std::io::Write;

            let mut writer = GzEncoder::new(Vec::new(), Compression::fast());
            if let Some(body) = response.body_bytes() {
                writer.write_all(body).unwrap();
                response.replace_body(writer.finish().unwrap());
                response.insert_header("Content-Encoding", "gzip");
            }
        }
    }

    #[cfg(feature = "log")]
    {
        log::trace!(
            "RESPONSE BODY LEN: {},\n RESPONSE HEADERS: {:#?}\n",
            response.body_len(),
            response.headers,
        );
    }

    response.send(conn);
}

fn read_to_vec<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}
