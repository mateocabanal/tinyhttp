#[derive(Copy, Clone)]
pub enum HTTP2FrameType {
    Data,
    Headers,
    Priority,
    RST_STREAM,
    Settings,
    PUSH_PROMISE,
    Ping,
    GO_AWAY,
    WINDOW_UPDATE,
    Continuation,
}

#[derive(Clone)]
pub struct HTTP2Frame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl HTTP2Frame {
    pub(crate) fn new() -> HTTP2Frame {
        HTTP2Frame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: Some(Vec::new()),
        }
    }

    pub fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    pub fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    pub fn get_frame_type_as_u8(&self) -> u8 {
        match self.frame_type.unwrap() {
            HTTP2FrameType::Data => 0,

            HTTP2FrameType::Headers => 1,

            HTTP2FrameType::Priority => 2,

            HTTP2FrameType::RST_STREAM => 3,

            HTTP2FrameType::Settings => 4,

            HTTP2FrameType::PUSH_PROMISE => 5,
            HTTP2FrameType::Ping => 6,

            HTTP2FrameType::GO_AWAY => 7,

            HTTP2FrameType::WINDOW_UPDATE => 8,
            HTTP2FrameType::Continuation => 9,
        }
    }

    pub fn frame_type(mut self, frame_type: u8) -> Self {
        let frame_type_enum = match frame_type {
            0 => HTTP2FrameType::Data,
            1 => HTTP2FrameType::Headers,
            2 => HTTP2FrameType::Priority,
            3 => HTTP2FrameType::RST_STREAM,
            4 => HTTP2FrameType::Settings,
            5 => HTTP2FrameType::PUSH_PROMISE,
            6 => HTTP2FrameType::Ping,
            7 => HTTP2FrameType::GO_AWAY,
            8 => HTTP2FrameType::WINDOW_UPDATE,
            9 => HTTP2FrameType::Continuation,
            _ => {
                log::error!("FRAME --> frame_type: doesn't exist!, returning a Data");
                HTTP2FrameType::Data
            }
        };
        self.frame_type = Some(frame_type_enum);
        self
    }
    pub fn payload(mut self, payload: Vec<u8>) -> Self {
        self.length = Some(payload.len());
        self.payload = Some(payload);
        self
    }

    pub fn stream_id(mut self, stream_id: u32) -> Self {
        self.stream_id = Some(stream_id);
        self
    }

    pub fn flags(mut self, flag: u8) -> Self {
        self.flags = Some(flag);
        self
    }

    pub fn to_vec(self) -> Vec<u8> {
        let mut frame: Vec<u8> = Vec::new();
        let frame_type = self.get_frame_type_as_u8();
        let payload = self.payload.unwrap();
        let payload_len = payload.len() as u32;
        let mut b_len = payload_len.to_be_bytes().to_vec();
        b_len.remove(0);
        frame.append(&mut b_len);
        frame.push(frame_type);
        frame.push(self.flags.unwrap());
        frame.append(&mut self.stream_id.unwrap().to_be_bytes().to_vec());
        frame = [frame, payload].concat();
        log::trace!("HTTP2_FRAME -> FRAME: {:#?}", frame);
        frame
    }
}
