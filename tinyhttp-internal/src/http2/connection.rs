use crate::http2::frame::*;

use super::frame::HTTP2Frame;

pub(crate) fn parse_buffer_to_frames(mut data_arr: Vec<u8>) -> Vec<Box<dyn HTTP2Frame>> {
    let mut http2_frames = Vec::new();
    let mut it = 0;
    if data_arr.starts_with(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n") {
        log::trace!("REMOVING PREFACE...: {:?}", data_arr);
        data_arr.drain(0..=23);
    }
    while it < data_arr.len() {
        let frame_len = 9 + u32::from_be_bytes(
            [vec![0u8], data_arr[it..=it + 2].to_vec()]
                .concat()
                .as_slice()
                .try_into()
                .unwrap(),
        );
        let frame =
            parse_data_frame(&data_arr[it..(it as u32 + frame_len).try_into().unwrap()]).unwrap();

         #[cfg(feature = "log")]
         log::trace!(
             "PARSED FRAME TYPE: {}",
             frame.get_frame_type_as_u8(),
         );

        http2_frames.push(frame);
        it += frame_len as usize;
    }

    http2_frames
}

pub(crate) fn parse_data_frame(
    data_arr: &[u8],
) -> Result<Box<dyn HTTP2Frame>, Box<dyn std::error::Error>> {
    let data = data_arr.to_vec();
    // log::trace!(
    //     "HTTP2 -> FIRST 24 bytes: {}",
    //     std::str::from_utf8(&data[0..=23]).unwrap()
    // );
    log::trace!("HTTP2 -> ENTIRE FRAME IN u8: {:#?}", data);
    //let length: u32 = u32::from_be_bytes(data[0..2].try_into()?);
    let length_vec: Vec<u8> = data[0..=2].to_vec();
    let length: u32 = u32::from_be_bytes([vec![0u8], length_vec].concat().as_slice().try_into()?);
    log::trace!("HTTP2 -> LENGTH: {}", length);
    let frame_type = data[3];

    log::trace!("HTTP2 -> frame_type: {}", frame_type);

    let flags = data[4];
    let end_stream = flags & 0x01 != 0;
    let end_padding = flags & 0x08 != 0;
    log::debug!(
        "HTTP2 -> FLAGS: {}, len of preface: {}",
        flags,
        b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n".len()
    );

    let reserved = data[5] & (1 << 7);

    let stream_id = u32::from_be_bytes(data[5..=8].try_into()?);
    log::debug!("HTTP2 -> STREAM ID: {}", stream_id);
    // if length != data.len() as u32 - 9 {
    //     return Err("Invalid frame length".into());
    // }
    // Check if the frame type is valid
    if frame_type > 0x12 {
        return Err("Invalid frame type".into());
    }
    // Check if the stream id is valid
    // if (stream_id & 0x80000000) == 0 {
    //     return Err("Invalid stream id".into());
    // }

    let payload = &data[9..length as usize + 9];

    //log::debug!("HTTP2 -> PAYLOAD:{}", std::str::from_utf8(payload).unwrap());

    match frame_type {
        0 => Ok(Box::new(
            DataFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        1 => Ok(Box::new(
            HeadersFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        2 => Ok(Box::new(
            PriorityFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        3 => Ok(Box::new(
            RSTFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        4 => Ok(Box::new(
            SettingsFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        5 => Ok(Box::new(
            PushFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        6 => Ok(Box::new(
            PingFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        7 => Ok(Box::new(
            GoAwayFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        8 => Ok(Box::new(
            WindowUpdateFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),
        9 => Ok(Box::new(
            ContinuationFrame::new()
                .frame_type(frame_type)
                .flags(flags)
                .payload(payload.to_vec())
                .stream_id(stream_id),
        )),

        _ => Err("not a frame type".into()),
    }
}
