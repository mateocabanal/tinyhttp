use std::{
    collections::HashMap,
    io::{Read, Write},
};

use crate::{
    http::read_stream,
    http2::{connection::parse_buffer_to_frames, frame::*},
};

macro_rules! unpack_octets_4 {
    // TODO: Get rid of this macro
    ($buf:expr, $offset:expr, $tip:ty) => {
        (($buf[$offset + 0] as $tip) << 24)
            | (($buf[$offset + 1] as $tip) << 16)
            | (($buf[$offset + 2] as $tip) << 8)
            | (($buf[$offset + 3] as $tip) << 0)
    };
}

#[derive(Clone, Debug)]
pub struct Response {
    pub headers: HashMap<String, String>,
    pub status_line: String,
    pub body: Option<Vec<u8>>,
    pub mime: Option<String>,
    pub http2: bool,
}

impl Response {
    pub fn new() -> Response {
        Response {
            headers: HashMap::new(),
            status_line: String::new(),
            body: None,
            mime: None,
            http2: false,
        }
    }

    #[allow(dead_code)]
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn status_line<P: Into<String>>(mut self, line: P) -> Self {
        self.status_line = line.into();
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

    pub(crate) fn send<P: Read + Write>(&self, sock: &mut P) {
        let line_bytes = self.status_line.as_bytes().to_vec();
        #[cfg(feature = "log")]
        log::debug!("res status line: {:#?}", self.status_line);

        let mut header_bytes = Vec::from_iter(
            self.headers
                .iter()
                .flat_map(|s| [s.0.as_bytes(), s.1.as_bytes()]),
        );
        let mut full_req = Vec::new();
        for i in line_bytes {
            full_req.push(i);
        }
        header_bytes.push(b"\r\n");
        for i in header_bytes {
            for j in i {
                full_req.push(*j);
            }
        }

        #[cfg(feature = "log")]
        log::debug!(
            "RESPONSE HEADERS (AFTER PARSE): {}",
            String::from_utf8(full_req.clone()).unwrap()
        );

        if let Some(i) = self.body.as_ref() {
            for j in i {
                full_req.push(*j);
            }
        }

        sock.write_all(&full_req).unwrap();
    }

    pub(crate) fn send_http_2<P: Read + Write>(&self, sock: &mut P) {
        // sock.write_all(
        //     b"HTTP/1.1 101 Switching Protocols \r\nConnection: Upgrade\r\nUpgrade: h2c\r\n\r\n",
        // )
        // .unwrap();
        let payload = SettingsFrame::build_payload();
        let frame = SettingsFrame::new()
            .frame_type(4)
            .flags(0)
            .payload(payload.to_vec())
            .stream_id(0);
        sock.write_all(&frame.to_vec()).unwrap();

        log::trace!("SENT INITIAL SETTINGS FRAME");

        //let mut buf = read_stream(sock);
        //buf.drain(0..=23);
        //let frame = parse_data_frame(&buf).unwrap();

        let mut term = false;

        while !term {
            let buf = read_stream(sock);

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
                    HTTP2FrameType::Headers => todo!(),
                    HTTP2FrameType::Priority => todo!(),
                    HTTP2FrameType::RST_STREAM => todo!(),
                    HTTP2FrameType::Settings => {
                        let payload = frame.get_payload().unwrap();
                        if payload.len() >= 6 {
                            let id: u16 = (u16::from(payload[0]) << 8) | u16::from(payload[1]);
                            //let id = u16::from_be_bytes(payload[0..=1].try_into().unwrap());
                            let value = u32::from_be_bytes(payload[2..=5].try_into().unwrap());

                            #[cfg(feature = "log")]
                            log::trace!("SETTINGS --> ID: {}, VALUE: {}", id, value);
                        }

                        #[cfg(feature = "log")]
                        log::trace!("SETTINGS FRAME RECV!, FLAG: {}", frame.get_flags());

                        if frame.get_flags() != 1 {
                            let settings_frame = SettingsFrame::new()
                                .frame_type(4)
                                .flags(1)
                                .stream_id(0)
                                .payload(Vec::from([3u8.to_be_bytes(), 100u8.to_be_bytes()]));
                            sock.write_all(&settings_frame.to_vec()).unwrap();
                            log::trace!("SENT SETTINGS FRAME!");
                        }
                    }
                    HTTP2FrameType::PUSH_PROMISE => todo!(),
                    HTTP2FrameType::Ping => todo!(),
                    HTTP2FrameType::GO_AWAY => {
                        let mut payload = frame.get_payload().unwrap();
                        payload[0] = payload[0] & 0xE;
                        let err_code = u32::from_be_bytes(payload[4..=7].try_into().unwrap());

                        #[cfg(feature = "log")]
                        log::trace!("GO_AWAY FRAME RECV!: {}", err_code);

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
                frames.remove(0);
            }
        }
    }
}
