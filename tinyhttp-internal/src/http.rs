use std::{
    cell::RefCell,
    fs::File,
    io::{self, BufReader},
    iter::FromIterator,
    path::Path,
    rc::Rc,
    time::Instant,
    vec,
};

#[cfg(not(feature = "async"))]
use std::io::{Read, Write};

#[cfg(feature = "sys")]
use flate2::{write::GzEncoder, Compression};

#[cfg(feature = "async")]
use async_std::io::{Read, Write};

#[cfg(feature = "async")]
use async_std::{
    io::{ReadExt, WriteExt},
    task::spawn,
};

use crate::{
    config::{Config, HttpListener},
    http2::{self},
    request::Request,
    response::Response,
};

#[cfg(not(feature = "async"))]
pub(crate) fn start_http(http: HttpListener) {
    for stream in http.get_stream() {
        let mut conn = stream.unwrap();
        let config = http.config.clone();

        if config.ssl && cfg!(feature = "ssl") {
            #[cfg(feature = "ssl")]
            let acpt = http.ssl_acpt.clone().unwrap();
            #[cfg(feature = "ssl")]
            http.pool.execute(move || match acpt.accept(conn) {
                Ok(mut s) => {
                    parse_request(&mut s, config);
                }
                Err(s) => {
                    #[cfg(feature = "log")]
                    log::error!("{}", s);
                }
            });
        } else if http.use_pool {
            http.pool.execute(move || {
                parse_request(&mut conn, config);
            });
        } else {
            parse_request(&mut conn, config);
        }

        //conn.write(b"HTTP/1.1 200 OK\r\n").unwrap();
    }
}

fn build_and_parse_req(buf: Vec<u8>) -> Request {
    #[cfg(feature = "log")]
    log::trace!("build_and_parse_req ~~> buf: {:?}", buf);

    if buf.starts_with(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n") || cfg!(debug_assertions) {
        return Request::new(vec![], vec![], vec![], None).set_http2(true);
    }
    let mut safe_http_index = buf.windows(2).enumerate();
    let status_line_index_opt = safe_http_index
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i);

    let status_line_index = if let Some(status) = status_line_index_opt {
        status
    } else {
        #[cfg(feature = "log")]
        log::error!("failed parsing status line!");

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
    while let Some(header_index) = safe_http_index
        .find(|(_, w)| matches!(*w, b"\r\n"))
        .map(|(i, _)| i + 2)
    {
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
    let str_status_line = Vec::from_iter(iter_status_line.split_whitespace());
    let status_line: Rc<Vec<String>> =
        Rc::new(str_status_line.iter().map(|i| String::from(*i)).collect());
    #[cfg(feature = "log")]
    log::debug!("{:#?}", status_line);
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
    Request::new(raw_body.to_vec(), headers, status_line.to_vec(), None)
}

fn build_res(req: &mut Request, config: &Config) -> Response {
    if req.get_http2() {
        let mut response = Response::new();
        response.http2 = true;
        return response;
    }
    let status_line = req.get_status_line();
    let req_path = Rc::new(RefCell::new(status_line[1].clone()));
    #[cfg(feature = "log")]
    log::trace!("build_res -> req_path: {}", req_path.borrow());

    match status_line[0].as_str() {
        "GET" => match config.get_routes(&mut req_path.borrow_mut()) {
            Some(route) => {
                #[cfg(feature = "log")]
                log::debug!("Found path in routes!");

                let req_new = if route.wildcard().is_some() {
                    let stat_line = &status_line[1];
                    let split = stat_line
                        .split(&(route.get_path().to_string() + "/"))
                        .last()
                        .unwrap();

                    req.set_wildcard(Some(split.into()))
                } else {
                    req
                };

                /*if route.is_ret_res() {
                    route.get_body_with_res().unwrap()(req_new.to_owned())
                } else {
                    Response::new()
                        .status_line("HTTP/1.1 200 OK\r\n")
                        .body(route.to_body(req_new.to_owned()))
                        .mime("text/plain")
                }*/

                route.to_res(req_new.to_owned())
            }

            None => match config.get_mount() {
                Some(old_path) => {
                    let path = old_path.to_owned() + &status_line[1];
                    if Path::new(&path).extension().is_none() && config.get_spa() {
                        let body = read_to_vec(&(old_path.to_owned() + "/index.html")).unwrap();
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
        "POST" => match config.post_routes(&mut req_path.borrow_mut()) {
            Some(route) => {
                #[cfg(feature = "log")]
                log::debug!("POST");

                let req_new = if route.wildcard().is_some() {
                    let stat_line = &status_line[1];

                    let split = stat_line
                        .split(&(route.get_path().to_string() + "/"))
                        .last()
                        .unwrap();

                    req.set_wildcard(Some(split.into()))
                } else {
                    req
                };

                route.to_res(req_new.to_owned())
            }

            None => Response::new()
                .status_line("HTTP/1.1 404 NOT FOUND\r\n")
                .body(b"<h1>404 Not Found</h1>".to_vec())
                .mime("text/html"),
        },

        _ => Response::new()
            .status_line("HTTP/1.1 404 NOT FOUND\r\n")
            .body(b"<h1>404 Not Found</h1>".to_vec())
            .mime("text/html"),
    }
}

fn parse_request<P: Read + Write>(conn: &mut P, config: Config) {
    let is_http2_check = conn.bytes().map(|x| x.unwrap());
    let buf = read_stream(conn);
    let mut request = build_and_parse_req(buf);

    let response = Rc::new(RefCell::new(build_res(&mut request, &config)));

    let res_brw = response.borrow_mut();
    if res_brw.http2 {
        res_brw.send_http_2(conn);
    }
    let mime = response.borrow_mut().mime.clone().unwrap();

    let inferred_mime = match infer::get(response.borrow_mut().body.as_ref().unwrap()) {
        Some(mime) => mime.mime_type().to_string(),
        None => mime,
    };

    match config.get_headers() {
        Some(vec) => {
            for (i, j) in vec.iter() {
                response
                    .borrow_mut()
                    .headers
                    .insert(i.to_string(), j.to_string());
            }
        }
        None => (),
    }

    response
        .borrow_mut()
        .headers
        .insert("Content-Type: ".to_string(), inferred_mime + "\r\n");

    response
        .borrow_mut()
        .headers
        .insert("X-:)-->: ".to_string(), "HEHEHE\r\n".to_string());

    let req_headers = request.get_headers();

    let comp = if req_headers.contains_key("accept-encoding") {
        let tmp_str: String = req_headers.get("accept-encoding").unwrap().to_owned();
        let res = tmp_str.split(',').map(|s| s.trim().to_string()).collect();

        #[cfg(feature = "log")]
        log::debug!("{:#?}", &res);

        res
    } else {
        Vec::new()
    };

    #[cfg(feature = "sys")]
    if config.get_gzip()
        && comp.contains(&"gzip".to_string())
        && req_headers.contains_key("accept-encoding")
    {
        #[cfg(feature = "log")]
        log::debug!("GZIP ENABLED!");
        let start: Instant = std::time::Instant::now();
        let body = response.borrow_mut().body.clone();
        let mut writer = GzEncoder::new(Vec::new(), Compression::default());
        writer.write_all(&body.unwrap()).unwrap();
        response.borrow_mut().body = Some(writer.finish().unwrap());
        response
            .borrow_mut()
            .headers
            .insert("Content-Encoding: ".to_string(), "gzip\r\n".to_string());
        #[cfg(feature = "log")]
        log::debug!("COMPRESS TOOK {} SECS", start.elapsed().as_secs());
    }

    let mut upgrade = String::from("");
    if req_headers.contains_key("connection") {
        upgrade = req_headers.get("connection").unwrap().to_string();

        let mut brw = response.borrow_mut();
        brw.headers
            .insert("Connection: ".to_owned(), "Upgrade\r\n".to_owned());

        brw.headers
            .insert("Upgrade: ".to_owned(), "h2c\r\n".to_owned());
        brw.send(conn);
        brw.send_http_2(conn);
        return;
    }

    #[cfg(feature = "log")]
    {
        let brw = response.borrow_mut();
        log::debug!(
            "RESPONSE BODY: {:#?},\n RESPONSE HEADERS: {:#?}\n",
            brw.body.as_ref().unwrap(),
            brw.headers,
        );
    }

    response.borrow_mut().send(conn);
}

pub fn read_to_vec<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    fn inner(path: &Path) -> io::Result<Vec<u8>> {
        let file = File::open(path).unwrap();
        let mut content: Vec<u8> = Vec::new();
        let mut reader = BufReader::new(file);
        reader.read_to_end(&mut content).unwrap();
        Ok(content)
    }
    inner(path.as_ref())
}

pub(crate) fn read_stream<P: Read>(stream: &mut P) -> Vec<u8> {
    let buffer_size = 512;
    let mut request_buffer = vec![];
    loop {
        let mut buffer = vec![0; buffer_size];
        match stream.read(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                } else if n < buffer_size {
                    request_buffer.append(&mut buffer[..n].to_vec());
                    break;
                } else {
                    request_buffer.append(&mut buffer);
                }
            }
            Err(e) => {
                println!("Error: Could not read string!: {}", e);
                std::process::exit(1);
            }
        }
    }

    request_buffer
}
