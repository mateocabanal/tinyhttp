#[derive(Clone)]
pub struct HTTP2Frame {
    length: Option<usize>,
    frame_type: Option<u8>,
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

    pub fn frame_type(mut self, frame_type: u8) -> Self {
        self.frame_type = Some(frame_type);
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

    pub fn to_vec(self) -> Vec<u8> {
        let mut frame: Vec<u8> = Vec::new();
        let payload = self.payload.unwrap();
        let payload_len = payload.len() as u32;
        let mut b_len = payload_len.to_be_bytes().to_vec();
        b_len.remove(0);
        frame.append(&mut b_len);
        frame.push(self.frame_type.unwrap());
        frame.push(self.flags.unwrap());
        frame.append(&mut self.stream_id.unwrap().to_be_bytes().to_vec());
        frame = [frame, payload].concat();
        log::trace!("HTTP2_FRAME -> FRAME: {:#?}", frame);
        frame
    }
}
