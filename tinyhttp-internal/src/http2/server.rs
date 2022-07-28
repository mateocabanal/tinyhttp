use std::io::Read;

use crate::config::HttpListener;

pub fn start_http2(mut http: HttpListener) {
    for stream in http.get_stream() {
        let mut conn = stream.unwrap();
        let mut buf = Vec::new();
        conn.read_to_end(&mut buf).unwrap();
    }
}
