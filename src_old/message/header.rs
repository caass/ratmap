use deku::prelude::*;

/// The message header contains metadata about the message.
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(endian = "big")]
pub struct MessageHeader {
    pub message_type: u8,

    /// The size of the payload in bytes.
    #[deku(bytes = 3)]
    pub payload_length: u32,

    /// Four-byte field that contains a timestamp of the message.
    pub timestamp: u32,

    /// Three-byte field that identifies the stream of the message.
    #[deku(bytes = 3)]
    pub stream_id: u32,
}
