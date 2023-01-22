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

    pub fn to_vec(self) -> Vec<u8> {
        let mut frame: Vec<u8> = Vec::new();
        let payload = self.payload.unwrap();
        let payload_len = payload.len() as u32;
        frame[0] = ((payload_len >> 16) & 0xFF) as u8;
        frame[1] = ((payload_len >> 8) & 0xFF) as u8;
        frame[2] = ((payload_len) & 0xFF) as u8;
        frame[3] = self.frame_type.unwrap();
        frame[4] = self.flags.unwrap();
        frame[5] = ((self.stream_id.unwrap() >> 24) & 0xFF) as u8;
        frame[6] = ((self.stream_id.unwrap() >> 16) & 0xFF) as u8;
        frame[7] = ((self.stream_id.unwrap() >> 8) & 0xFF) as u8;
        frame[8] = (self.stream_id.unwrap() & 0xFF) as u8;
        frame = [frame, payload].concat();
        frame
    }
}
