use std::net::TcpListener;

use tinyhttp::prelude::*;

#[get("/")]
fn get(_req: Request) -> &'static str {
    "Hello, there!\n"
}

#[post("/")]
fn post(body: Request) -> String {
    format!("Hello, {:?}\n", body.get_raw_body())
}

fn main() {
    let socket = TcpListener::bind(":::9001").unwrap();
    let routes = Routes::new(vec![get(), post()]);
    let config = Config::new().routes(routes);
    let http = HttpListener::new(socket, config);

    http.start();
}
