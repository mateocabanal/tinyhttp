use criterion::{criterion_group, criterion_main, Criterion};
use std::io::{Read, Write};

use std::sync::Arc;
use tinyhttp::prelude::*;
use tinyhttp_internal::http::parse_request;

struct RwWrapper<'a, T> {
    pub read: &'a [u8],
    pub write: T,
}

impl<'a, T> Read for RwWrapper<'a, T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.read.read_to_end(buf)
    }
}

impl<'a, T> Write for RwWrapper<'a, T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write.write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write.write_all(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.write.flush()
    }
}

impl<'a, T> RwWrapper<'a, T>
where
    T: Write,
{
    fn new(read: &'a [u8], write: T) -> Self {
        RwWrapper { read, write }
    }
}

/// Struct containing data on a single request.
///
/// parsed_body which is a Option<String> that can contain the body as a String
///
/// body is used when the body of the request is not a String

#[get("/helloworld")]
fn get() -> &'static str {
    "got it"
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let http = "GET /helloworld HTTP/1.1\r\nAccept-Content: text/plain\r\n\r\n".as_bytes();

    let conf = Arc::new(Config::new().routes(Routes::new(vec![get()])));
    let buffer = Vec::with_capacity(16384);
    let mut read_write = RwWrapper::new(http, buffer);
    c.bench_function("Parse http request", move |b| {
        b.iter(|| {
            parse_request(&mut read_write, conf.clone());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
