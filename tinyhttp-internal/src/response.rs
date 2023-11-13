use std::{
    collections::HashMap,
    io::{Read, Write},
};

#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Clone, Debug)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub status_line: String,
    pub body: Option<Vec<u8>>,
    pub mime: Option<String>,
    pub http2: bool,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a str> for Response {
    fn from(value: &'a str) -> Self {
        Response::new().body(value.into()).mime("text/plain").status_line("HTTP/1.1 200 OK")
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Response::new().body(value.into_bytes()).mime("text/plain").status_line("HTTP/1.1 200 OK")
    }
}

impl From<Vec<u8>> for Response {
    fn from(value: Vec<u8>) -> Self {
        Response::new().body(value).mime("application/octet-stream").status_line("HTTP/1.1 200 OK")
    }
}

impl Response {
    pub fn new() -> Response {
        Response {
            headers: HashMap::new(),
            status_line: String::new(),
            body: None,
            mime: Some(String::from("HTTP/1.1 200 OK")),
            http2: false,
        }
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn status_line<P: Into<String>>(mut self, line: P) -> Self {
        let line_str = line.into();
        self.status_line = line_str.trim().to_string() + "\r\n";
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn mime<P>(mut self, mime: P) -> Self
    where
        P: Into<String>,
    {
        self.mime = Some(mime.into());
        self
    }

    #[cfg(not(feature = "async"))]
    pub(crate) fn send<P: Read + Write>(&self, sock: &mut P) {
        let line_bytes = self.status_line.as_bytes();
        #[cfg(feature = "log")]
        log::trace!("res status line: {:#?}", self.status_line);

        let mut header_bytes: Vec<u8> = self
            .headers
            .iter()
            .flat_map(|(i, j)| [i.as_bytes(), j.as_bytes()].concat())
            .collect();

        header_bytes.extend(b"\r\n");

        #[cfg(all(feature = "log", debug_assertions))]
        {
            log::trace!(
                "HEADER AS STR: {}",
                String::from_utf8(header_bytes.clone()).unwrap()
            );
            log::trace!(
                "STATUS LINE AS STR: {}",
                std::str::from_utf8(line_bytes).unwrap()
            );
        };

        let full_req: &[u8] = &[
            line_bytes,
            header_bytes.as_slice(),
            self.body.as_ref().unwrap(),
        ]
        .concat();

        sock.write_all(full_req).unwrap();
    }

<<<<<<< HEAD
    pub(crate) fn send_http_2<P: Read + Write>(&self, sock: &mut P) {
        // sock.write_all(
        //     b"HTTP/1.1 101 Switching Protocols \r\nConnection: Upgrade\r\nUpgrade: h2c\r\n\r\n",
        // )
        // .unwrap();

        // Sets max_concurrent_streams to 1
        let payload = SettingsFrame::build_payload();
        let frame = SettingsFrame::new()
            .frame_type(4)
            .flags(0)
            .payload(payload)
            .stream_id(0);
        sock.write_all(&frame.to_vec()).unwrap();

        log::trace!("SENT INITIAL SETTINGS FRAME");

        //let mut buf = read_stream(sock);
        //buf.drain(0..=23);
        //let frame = parse_data_frame(&buf).unwrap();
=======
    #[cfg(feature = "async")]
    pub(crate) async fn send<P: AsyncReadExt + AsyncWriteExt + Unpin>(&self, sock: &mut P) {
        let line_bytes = self.status_line.as_bytes();
        #[cfg(feature = "log")]
        log::trace!("res status line: {:#?}", self.status_line);

        let mut header_bytes: Vec<u8> = self
            .headers
            .iter()
            .flat_map(|s| [s.0.as_bytes(), s.1.as_bytes()].concat())
            .collect();
>>>>>>> origin/main

        header_bytes.extend(b"\r\n");

        #[cfg(all(feature = "log", debug_assertions))]
        {
            log::trace!(
                "HEADER AS STR: {}",
                String::from_utf8(header_bytes.clone()).unwrap()
            );
            log::trace!(
                "STATUS LINE AS STR: {}",
                std::str::from_utf8(line_bytes).unwrap()
            );
        };

<<<<<<< HEAD
            #[cfg(feature = "log")]
            //log::trace!("BUFFER BEFORE parse_buffer_to_frames: {:?}", buf);
            let mut frames = parse_buffer_to_frames(buf);

            for frame in frames.clone() {
                #[cfg(feature = "log")]
                //log::trace!("frames: {:#?}", frames);
                match frame.get_frame_type() {
                    HTTP2FrameType::Data => {
                        #[cfg(feature = "log")]
                        log::debug!("data frame found");

                        todo!()
                    }
                    HTTP2FrameType::Headers => {
                        log::debug!("header frame found");
                    },
                    HTTP2FrameType::Priority => todo!(),
                    HTTP2FrameType::RST_STREAM => todo!(),
                    HTTP2FrameType::Settings => {
                        let payload = frame.get_payload().unwrap();
                        let mut pos = 0;
                        while pos < payload.len() {
                            let id: u16 = (u16::from(payload[pos]) << 8) | u16::from(payload[pos + 1]);
                            //let id = u16::from_be_bytes(payload[0..=1].try_into().unwrap());
                            let value = u32::from_be_bytes(payload[pos + 2..=pos + 5].try_into().unwrap());

                            #[cfg(feature = "log")]
                            log::trace!("SETTINGS --> ID: {}, VALUE: {}", id, value);
                            pos += 6;
                        }

                        #[cfg(feature = "log")]
                        log::trace!("SETTINGS FRAME RECV!, FLAG: {}", frame.get_flags());

                        if frame.get_flags() != 1 {
                            let settings_frame = SettingsFrame::new()
                                .frame_type(4)
                                .flags(0)
                                .stream_id(0)
                                .payload(vec![]);

                            let settings_vec = settings_frame.to_vec();
                            log::trace!("sending settings frame: {:?}", &settings_vec);
                            sock.write_all(&settings_vec).unwrap();
                            log::trace!("SENT SETTINGS FRAME!");
                        } else {
                            log::trace!("client ACK'd!");
                        }

                    }
                    HTTP2FrameType::PUSH_PROMISE => todo!(),
                    HTTP2FrameType::Ping => {
                        let mut ping_frame = PingFrame::new();
                        let ping_frame = ping_frame.flags(frame.get_flags());
                        let ping_frame = ping_frame.payload(frame.get_payload().unwrap());
                        let ping_frame = ping_frame.flags(0x01);
                        sock.write_all(&ping_frame.to_vec()).unwrap();
                        log::trace!("SENT PING FRAME!");
                    }
                    HTTP2FrameType::GO_AWAY => {
                        let mut payload = frame.get_payload().unwrap();
                        payload[0] = payload[0] & 0xE;
                        let debug_info = &payload[7..];
                        let err_code = u32::from_be_bytes(payload[4..=7].try_into().unwrap());

                        #[cfg(feature = "log")]
                        log::trace!("GO_AWAY FRAME RECV!: {}", err_code);
                        log::trace!("GO_AWAY: debug info: {}", String::from_utf8_lossy(debug_info));

                        term = true;
                    }
                    HTTP2FrameType::WINDOW_UPDATE => {
                        let payload = frame.get_payload().unwrap();
                        if payload.len() >= 4 {
                            let window_inc = u32::from_be_bytes(payload[0..=3].try_into().unwrap());

                            #[cfg(feature = "log")]
                            log::trace!("WINDOW_UPDATE FRAME RECV!, window_inc: {}", window_inc);
                        }
                    }
                    HTTP2FrameType::Continuation => todo!(),
                }
            }
        }
=======
        let full_req: &[u8] = &[
            line_bytes,
            header_bytes.as_slice(),
            self.body.as_ref().unwrap(),
        ]
        .concat();

        sock.write_all(full_req).await.unwrap();
>>>>>>> origin/main
    }
}
