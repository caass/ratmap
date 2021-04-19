use deku::prelude::*;

/// The client or the server sends this message to notify the peer about
/// the user control events. The implementation is based on sections
/// [6.2] and [7.1.7] of the spec.
///
/// [6.2]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=23
/// [7.1.7]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=27
#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(type = "u16", endian = "big")]
pub enum Event {
    /// The server sends this event to notify the client
    /// that a stream has become functional and can be
    /// used for communication.
    ///
    /// By default, this event
    /// is sent on ID 0 after the application connect
    /// command is successfully received from the
    /// client.
    ///
    /// The event data is 4-byte and represents
    /// the stream ID of the stream that became
    /// functional.
    #[deku(id = "0")]
    StreamBegin { stream_id: u32 },

    /// The server sends this event to notify the client
    /// that the playback of data is over as requested
    /// on this stream. No more data is sent without
    /// issuing additional commands. The client discards
    /// the messages received for the stream. The
    /// 4 bytes of event data represent the ID of the
    /// stream on which playback has ended.
    #[deku(id = "1")]
    StreamEof { stream_id: u32 },

    /// The server sends this event to notify the client
    /// that there is no more data on the stream. If the
    /// server does not detect any message for a time
    /// period, it can notify the subscribed clients
    /// that the stream is dry. The 4 bytes of event
    /// data represent the stream ID of the dry stream.
    #[deku(id = "2")]
    StreamDry { stream_id: u32 },

    /// The client sends this event to inform the server
    /// of the buffer size (in milliseconds) that is
    /// used to buffer any data coming over a stream.
    /// This event is sent before the server starts
    /// processing the stream. The first 4 bytes of the
    /// event data represent the stream ID and the next
    /// 4 bytes represent the buffer length, in milliseconds.
    #[deku(id = "3")]
    SetBufferLength { stream_id: u32, buffer_length: u32 },

    /// The server sends this event to notify the client
    /// that the stream is a recorded stream. The
    /// 4 bytes event data represent the stream ID of
    /// the recorded stream.
    #[deku(id = "4")]
    StreamIsRecorded { stream_id: u32 },

    /// The server sends this event to test whether the
    /// client is reachable. Event data is a 4-byte
    /// timestamp, representing the local server time
    /// when the server dispatched the command. The
    /// client responds with PingResponse on receiving
    /// MsgPingRequest.
    #[deku(id = "6")]
    PingRequest { timestamp: u32 },

    /// The client sends this event to the server in
    /// response to the ping request. The event data is
    /// a 4-byte timestamp, which was received with the
    /// PingRequest request.
    #[deku(id = "7")]
    PingResponse { timestamp: u32 },
}
