use std::{
    io::{self, BufRead, BufReader},
    net::TcpStream,
    path::Path,
    sync::Arc,
};

use std::{fs::File, io::Read};

use crate::{
    config::{Config, HttpListener},
    headers::HeaderMap,
    request::{Request, RequestError},
    response::Response,
};

#[cfg(feature = "sys")]
use flate2::{write::GzEncoder, Compression};
use unicase::Ascii;

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

fn build_and_parse_req<P: Read>(conn: &mut P) -> Result<Request, RequestError> {
    let mut buf_reader = BufReader::new(conn);
    let mut status_line_str = String::new();

    buf_reader.read_line(&mut status_line_str).unwrap();
    status_line_str.drain(status_line_str.len() - 2..status_line_str.len());

    #[cfg(feature = "log")]
    log::trace!("STATUS LINE: {:#?}", status_line_str);

    let iter = buf_reader.fill_buf().unwrap();
    let header_end_idx = iter
        .windows(4)
        .position(|w| matches!(w, b"\r\n\r\n"))
        .unwrap();

    #[cfg(feature = "log")]
    log::trace!("Body starts at {}", header_end_idx);
    let headers_buf = iter[..header_end_idx + 2].to_vec();

    buf_reader.consume(header_end_idx + 4); // Add 4 bytes since header_end_idx does not count
                                            // \r\n\r\n

    let mut headers = HeaderMap::new();
    let mut headers_index = 0;

    let mut headers_buf_iter = headers_buf.windows(2).enumerate();

    //Sort through all request headers
    while let Some(header_index) = headers_buf_iter
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i)
    {
        #[cfg(feature = "log")]
        log::trace!("header index: {}", header_index);

        let header = std::str::from_utf8(&headers_buf[headers_index..header_index]).unwrap();

        if header.is_empty() {
            break;
        }
        #[cfg(feature = "log")]
        log::trace!("HEADER: {:?}", header);

        headers_index = header_index + 2;

        let mut colon_split = header.splitn(2, ':');
        headers.set(
            colon_split.next().unwrap(),
            colon_split.next().unwrap().trim(),
        );
    }

    let body_len = headers
        .get(Ascii::new("Content-Length".to_string()))
        .map(|str| str.parse::<usize>().unwrap())
        .unwrap_or(0usize);

    let mut raw_body = vec![0; body_len];
    buf_reader.read_exact(&mut raw_body).unwrap();

    Ok(Request::new(
        raw_body,
        headers,
        status_line_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect(),
        None,
    ))
}

fn build_res(mut req: Request, config: &Config, sock: &mut TcpStream) -> Response {
    let status_line = req.get_status_line();
    #[cfg(feature = "log")]
    log::trace!("build_res -> req_path: {}", status_line[1]);

    match status_line[0].as_str() {
        "GET" => match config.get_routes(&status_line[1]) {
            Some(route) => {
                #[cfg(feature = "log")]
                log::trace!("Found path in routes!");

                if route.wildcard().is_some() {
                    let stat_line = &status_line[1];
                    let split = stat_line
                        .split(&(route.get_path().to_string() + "/"))
                        .last()
                        .unwrap();

                    req.set_wildcard(Some(split.into()));
                };

                route.to_res(req, sock)
            }

            None => match config.get_mount() {
                Some(old_path) => {
                    let path = old_path.to_owned() + &status_line[1];
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
                                .status_line("HTTP/1.1 200 OK\r\n")
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
        "POST" => match config.post_routes(&status_line[1]) {
            Some(route) => {
                #[cfg(feature = "log")]
                log::debug!("POST");

                if route.wildcard().is_some() {
                    let stat_line = &status_line[1];

                    let split = stat_line
                        .split(&(route.get_path().to_string() + "/"))
                        .last()
                        .unwrap();

                    req.set_wildcard(Some(split.into()));
                };

                route.to_res(req, sock)
            }

            None => Response::new()
                .status_line("HTTP/1.1 404 NOT FOUND\r\n")
                .body(b"<h1>404 Not Found</h1>".to_vec())
                .mime("text/html"),
        },

        _ => Response::new()
            .status_line("HTTP/1.1 404 NOT FOUND\r\n")
            .body(b"<h1>Unkown Error Occurred</h1>".to_vec())
            .mime("text/html"),
    }
}

pub fn parse_request(conn: &mut TcpStream, config: Arc<Config>) {
    let request = build_and_parse_req(conn);

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
                .send(conn);

            return;
        }
    };

    /*#[cfg(feature = "middleware")]
    if let Some(req_middleware) = config.get_request_middleware() {
        req_middleware.lock().unwrap()(&mut request);
    };*/

    let req_headers = request.get_headers();
    let _comp = if config.get_gzip() {
        if req_headers.contains("Accept-Encoding") {
            let tmp_str = req_headers.get("Accept-Encoding").unwrap();
            let res: Vec<&str> = tmp_str.split(',').map(|s| s.trim()).collect();

            #[cfg(feature = "log")]
            log::trace!("{:#?}", &res);

            res.contains(&"gzip")
        } else {
            false
        }
    } else {
        false
    };

    let mut response = build_res(request, &config, conn);
    if response.manual_override {
        conn.shutdown(std::net::Shutdown::Both).unwrap();
        return;
    }

    match response.mime {
        Some(ref t) => {
            response
                .headers
                .insert("Content-Type".to_string(), t.to_owned());
        }
        None => {
            if let Some(body) = &response.body {
                let mime = infer::get(body)
                    .map(|mime| mime.mime_type())
                    .unwrap_or("text/plain");

                response
                    .headers
                    .insert("Content-Type".to_string(), mime.to_string());
            }
        }
    }

    if let Some(config_headers) = config.get_headers() {
        response.headers.extend(
            config_headers
                .iter()
                .map(|(i, j)| (i.to_owned(), j.to_owned())),
        );
    }

    response.headers.extend([(
        "tinyhttp".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    )]);

    // Only check for 'accept-encoding' header
    // when compression is enabled

    #[cfg(feature = "sys")]
    {
        if _comp {
            use std::io::Write;
            let mut writer = GzEncoder::new(Vec::new(), Compression::default());
            writer.write_all(response.body.as_ref().unwrap()).unwrap();
            response.body = Some(writer.finish().unwrap());
            response
                .headers
                .insert("Content-Encoding".to_string(), "gzip".to_string());
        }
    }

    #[cfg(feature = "log")]
    {
        log::trace!(
            "RESPONSE BODY: {:#?},\n RESPONSE HEADERS: {:#?}\n",
            response.body.as_ref().unwrap(),
            response.headers,
        );
    }

    /*#[cfg(feature = "middleware")]
    if let Some(middleware) = config.get_response_middleware() {
        middleware.lock().unwrap()(res_brw.deref_mut());
    }*/

    response.send(conn);
}

fn read_to_vec<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    fn inner(path: &Path) -> io::Result<Vec<u8>> {
        let file = File::open(path).unwrap();
        let mut content: Vec<u8> = Vec::new();
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut content).unwrap();
        Ok(content)
    }
    inner(path.as_ref())
}
