use dyn_clone::DynClone;

#[derive(Copy, Clone)]
pub enum HTTP2FrameType {
    Data = 0,
    Headers = 1,
    Priority = 2,
    RST_STREAM = 3,
    Settings = 4,
    PUSH_PROMISE = 5,
    Ping = 6,
    GO_AWAY = 7,
    WINDOW_UPDATE = 8,
    Continuation = 9,
}

pub trait HTTP2Frame: DynClone {
    fn get_flags(&self) -> u8;

    fn get_payload(&self) -> Option<Vec<u8>>;

    fn get_frame_type(&self) -> HTTP2FrameType;

    fn get_frame_type_as_u8(&self) -> u8;
    fn to_vec(self) -> Vec<u8>;
}

impl Clone for Box<dyn HTTP2Frame> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

#[derive(Clone)]
pub(crate) struct DataFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl DataFrame {
    pub(crate) fn new() -> DataFrame {
        DataFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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
}

impl HTTP2Frame for DataFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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

#[derive(Clone)]
pub(crate) struct HeadersFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl HeadersFrame {
    pub(crate) fn new() -> HeadersFrame {
        HeadersFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for HeadersFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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
#[derive(Clone)]
pub(crate) struct PriorityFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl PriorityFrame {
    pub(crate) fn new() -> PriorityFrame {
        PriorityFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for PriorityFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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

#[derive(Clone)]
pub(crate) struct RSTFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl RSTFrame {
    pub(crate) fn new() -> RSTFrame {
        RSTFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for RSTFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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

#[derive(Clone)]
pub(crate) struct SettingsFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl SettingsFrame {
    pub(crate) fn new() -> SettingsFrame {
        SettingsFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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
        frame.append(&mut frame_type.to_be_bytes().to_vec());
        frame.append(&mut self.flags.unwrap().to_be_bytes().to_vec());
        frame.append(&mut self.stream_id.unwrap().to_be_bytes().to_vec());
        frame = [frame, payload].concat();
        log::trace!("HTTP2_FRAME -> FRAME: {:#?}", frame);
        frame
    }

    pub fn build_payload() -> Vec<u8> {
        let mut payload = Vec::new();
        let id: [u8; 2] = 3u16.to_be_bytes();
        let value: [u8; 4] = 1u32.to_be_bytes();
        payload = [id.as_slice(), value.as_slice()].concat();
        payload
    }
}

impl HTTP2Frame for SettingsFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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

#[derive(Clone)]
pub(crate) struct PushFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl PushFrame {
    pub(crate) fn new() -> PushFrame {
        PushFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for PushFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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

#[derive(Clone)]
pub(crate) struct PingFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl PingFrame {
    pub(crate) fn new() -> PingFrame {
        PingFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for PingFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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
#[derive(Clone)]
pub(crate) struct GoAwayFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl GoAwayFrame {
    pub(crate) fn new() -> GoAwayFrame {
        GoAwayFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for GoAwayFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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
#[derive(Clone)]
pub(crate) struct WindowUpdateFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl WindowUpdateFrame {
    pub(crate) fn new() -> WindowUpdateFrame {
        WindowUpdateFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for WindowUpdateFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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
#[derive(Clone)]
pub(crate) struct ContinuationFrame {
    length: Option<usize>,
    frame_type: Option<HTTP2FrameType>,
    flags: Option<u8>,
    reserved: Option<u8>,
    stream_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl ContinuationFrame {
    pub(crate) fn new() -> ContinuationFrame {
        ContinuationFrame {
            length: None,
            frame_type: None,
            flags: Some(0),
            reserved: Some(0),
            stream_id: Some(0),
            payload: None,
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

impl HTTP2Frame for ContinuationFrame {
    fn get_flags(&self) -> u8 {
        self.flags.unwrap()
    }

    fn get_payload(&self) -> Option<Vec<u8>> {
        self.payload.clone()
    }

    fn get_frame_type(&self) -> HTTP2FrameType {
        self.frame_type.unwrap()
    }

    fn get_frame_type_as_u8(&self) -> u8 {
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
    fn to_vec(self) -> Vec<u8> {
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
