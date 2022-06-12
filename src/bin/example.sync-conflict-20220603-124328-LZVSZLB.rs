use std::net::TcpListener;

use tinyhttp::tinyhttp_codegen::*;
use tinyhttp::tinyhttp_internal::config::*;
use tinyhttp::tinyhttp_internal::request::Request;

#[get("/")]
fn get() -> &'static str {
    "Hello, there!\n"
}

#[post("/")]
fn post(body: Request) -> Vec<u8> {
    let parsed_body = body.get_parsed_body().unwrap();
    format!("Hello, {}\n", parsed_body).into()
}

fn main() {
    let socket = TcpListener::bind(":::9001").unwrap();
    let routes = Routes::new(vec![get(), post()]);
    let config = Config::new().routes(routes);
    let http = HttpListener::new(socket, config);

    http.start();
}
