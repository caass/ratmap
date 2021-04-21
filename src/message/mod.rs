mod header;
mod payload;

use deku::prelude::*;
use futures::{Sink, Stream};

pub use header::MessageHeader;
pub use payload::MessageData;

/// The server and the client send RTMP messages over the network to
/// communicate with each other. The messages could include audio,
/// video, data, or any other messages.
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct Message {
    pub header: MessageHeader,
    #[deku(ctx = "header.message_type, header.payload_length, header.stream_id, header.timestamp")]
    pub data: MessageData,
}
