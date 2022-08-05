use std::io::{Read, Write};

use crate::config::HttpListener;

pub fn start_http2<S: Read + Write>(mut socket: S) {
    let mut buf = Vec::new();
    socket.read_to_end(&mut buf).unwrap();
    println!("HTTP2!");
}
