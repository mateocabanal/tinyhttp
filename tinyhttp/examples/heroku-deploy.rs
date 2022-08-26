#[warn(unused)]
use std::{net::TcpListener, process::Command};

use tinyhttp::prelude::*;

#[get("/api")]
fn api_get() -> &'static str {
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

#[get("/update_html")]
fn update_html() -> &'static str {
    init_html();
    "OK"
}

fn init_html() {
    Command::new("wget")
        .arg("https://github.com/mateocabanal/tinyhttp-heroku-html/archive/refs/heads/main.zip")
        .output()
        .unwrap();

    println!(
        "unzip output: {}",
        String::from_utf8(
            Command::new("unzip")
                .arg("main.zip")
                .output()
                .unwrap()
                .stdout
        )
        .unwrap()
    );

    println!("UNZIPPED!");
}

fn main() {
    init_html();
    let socket = TcpListener::bind(":::".to_owned() + &std::env::var("PORT").unwrap()).unwrap();
    let routes = Routes::new(vec![
        api_get(),
        update_html(),
        post(),
        post_without_args(),
        get_wildcard(),
        post_wildcard(),
        post_return_vec(),
    ]);
    let config = Config::new()
        .routes(routes)
        .gzip(true)
        .mount_point("./tinyhttp-heroku-html-main");
    let http = HttpListener::new(socket, config);

    http.start();
}
