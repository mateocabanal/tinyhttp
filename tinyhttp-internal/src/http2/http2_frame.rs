pub struct HTTP2Frame {
    length: Option<u32>,
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
            flags: None,
            reserved: None,
            stream_id: None,
            payload: None,
        }
    }
}
