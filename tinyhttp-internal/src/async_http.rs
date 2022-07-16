use std::{
    fs::File,
    io::{self, BufReader},
    iter::FromIterator,
    path::Path,
    rc::Rc,
    time::Instant,
    vec,
};

use async_std::io::{Read, Write};
use async_std::io::{ReadExt, WriteExt};
use async_std::task::spawn;

#[cfg(feature = "sys")]
use flate2::write::GzEncoder;
#[cfg(feature = "sys")]
use flate2::Compression;
use futures::StreamExt;

use crate::{
    config::{Config, HttpListener},
    request::{self, Request},
};

#[cfg(feature = "async")]
pub async fn start_http(http: HttpListener) {
    http.get_stream()
        .for_each_concurrent(None, |tcp_stream| async {
            let mut conn = tcp_stream.unwrap();
            let config = http.config.clone();

            #[cfg(feature = "log")]
            log::trace!("CALLING spawn");

            spawn(async move {
                parse_request(&mut conn, config).await;
            });

            //conn.write(b"HTTP/1.1 200 OK\r\n").unwrap();
        })
        .await;
}

fn build_and_parse_req(buf: Vec<u8>) -> Request {
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
    request::Request::new(
        raw_body.to_vec(),
        headers.clone(),
        status_line.to_vec(),
        None,
    )
}

async fn parse_request<P: Read + Write + std::marker::Unpin>(conn: &mut P, config: Config) {
    let buf = read_stream(conn).await;
    let request = build_and_parse_req(buf);
    let status_line = request.get_status_line().clone();
    let mut res_headers: Vec<String> = Vec::new();

    let (c_status_line, mut body, mime) = match status_line[0].as_str() {
        "GET" => match config.get_routes(status_line[1].clone()) {
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
            io::Write::write_all(&mut writer, &body).unwrap();
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
            io::Read::read_to_end(&mut compressor, &mut body).unwrap();
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
    match conn.write_all(&res).await {
        Ok(_) => {
            #[cfg(feature = "log")]
            log::debug!("WROTE response");
        }

        Err(_) => {
            #[cfg(feature = "log")]
            log::warn!("ERROR on response!")
        }
    }
}

async fn read_stream<P: Read + std::marker::Unpin>(stream: &mut P) -> Vec<u8> {
    let buffer_size = 512;
    let mut request_buffer = vec![];
    loop {
        let mut buffer = vec![0; buffer_size];
        match stream.read(&mut buffer).await {
            Ok(n) => {
                if n == 0 {
                    break;
                } else {
                    if n < buffer_size {
                        request_buffer.append(&mut buffer[..n].to_vec());
                        break;
                    } else {
                        request_buffer.append(&mut buffer);
                    }
                }
            }
            Err(_) => println!("Error: Could not read string!"),
        }
    }

    request_buffer
}

pub fn read_to_vec<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    fn inner(path: &Path) -> io::Result<Vec<u8>> {
        let file = File::open(path).unwrap();
        let mut content: Vec<u8> = Vec::new();
        let mut reader = BufReader::new(file);
        io::Read::read_to_end(&mut reader, &mut content).unwrap();
        Ok(content)
    }
    inner(path.as_ref())
}
