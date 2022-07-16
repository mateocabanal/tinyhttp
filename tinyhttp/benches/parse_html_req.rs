use std::{net::TcpListener, thread};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tinyhttp::prelude::*;

#[get("/")]
fn get(_req: Request) -> &'static str {
  "Hello Bench!\n"
}

fn bench(c: &mut Criterion) {
  let sock = TcpListener::bind(":::9156").unwrap();
  let conf = Config::new().routes(Routes::new(vec![get()]));
  let http = HttpListener::new(sock, conf);
  thread::spawn( move || {
    http.start();
  });
  c.bench_function("measuring response time", |b| b.iter(|| reqwest::blocking::get("localhost:9156")));
}

criterion_group!(benches, bench);
criterion_main!(benches);