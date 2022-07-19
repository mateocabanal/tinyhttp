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

fn main() {
    let socket = TcpListener::bind(":::9001").unwrap();
    let routes = Routes::new(vec![get(), post(), post_without_args()]);
    let config = Config::new().routes(routes).gzip(false);
    let http = HttpListener::new(socket, config);

    http.start();
}
