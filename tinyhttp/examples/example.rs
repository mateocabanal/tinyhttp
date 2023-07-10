use std::net::TcpListener;

use tinyhttp::prelude::*;

#[get("/")]
fn get() -> &'static str {
    "Hello, there!\n"
}

#[post("/")]
fn post(body: Request) -> String {
    format!("Hello, {:?}\n", body.get_parsed_body().unwrap())
}

#[post("/w")]
fn post_without_args() -> &'static str {
    "HEHE\n"
}

#[get("/wildcard/:")]
fn get_wildcard(req: Request) -> String {
    let wildcard = req.get_wildcard().unwrap();
    format!("Hello, {}\n", wildcard)
}

#[post("/wildcard/:")]
fn post_wildcard(req: Request) -> String {
    let wildcard = req.get_wildcard().unwrap();
    format!("Hello, {}\n", wildcard)
}

#[post("/test/returning/vec")]
fn post_return_vec() -> Vec<u8> {
    b"Hello World!".to_vec()
}

#[get("/return_res")]
fn get_return_res(res: Request) -> Response {
    if res.get_status_line()[1] == "/return_res" {
        Response::new()
            .status_line("HTTP/1.1 200 OK\r\n")
            .body(b"Hello, from response!\r\n".to_vec())
            .mime("text/plain")
    } else {
        unreachable!()
    }
}

fn main() {
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();

    let middleware = |res: &mut Response| {
        res.headers.insert("X-MIDDLEWARE: ".to_string(), "SOMETHING\r\n".to_string());
    };

    let socket = TcpListener::bind(":::9001").unwrap();
    let routes = Routes::new(vec![
        get(),
        post(),
        post_without_args(),
        get_wildcard(),
        post_wildcard(),
        post_return_vec(),
    ]);
    let config = Config::new().routes(routes).gzip(true).middleware(middleware);
    let http = HttpListener::new(socket, config);

    http.start();
}
