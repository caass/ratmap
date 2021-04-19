mod aggregate;
mod amf;
// mod command;
mod protocol_control;
// mod shared_object;
mod user_control;

use deku::prelude::*;

pub use self::amf::AmfValue;
pub use aggregate::Aggregate;
// use command::CommandMessage;
pub use protocol_control::ProtocolControlMessage;
pub use user_control::Event;

/// The other part of the message is the payload, which is the actual
/// data contained in the message. For example, it could be some audio
/// samples or compressed video data.
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(
    ctx = "message_type: u8, payload_length: u32, message_stream_id: u32, message_timestamp: u32",
    id = "message_type"
)]
pub enum MessageData {
    /// RTMP Chunk Stream uses message type IDs 1, 2, 3, 5, and 6 for
    /// protocol control messages. These messages contain information needed
    /// by the RTMP Chunk Stream protocol.
    /// These protocol control messages MUST have message stream ID 0 (known
    /// as the control stream) and be sent in chunk stream ID 2. Protocol
    /// control messages take effect as soon as they are received; their
    #[deku(id_pat = "1 | 2 | 3 | 5 | 6")]
    ProtocolControl(#[deku(ctx = "message_type")] ProtocolControlMessage),

    /// RTMP uses message type ID 4 for User Control messages. These
    /// messages contain information used by the RTMP streaming layer.
    ///
    /// User Control messages SHOULD use message stream ID 0 (known as the
    /// control stream) and, when sent over RTMP Chunk Stream, be sent on
    /// chunk stream ID 2. User Control messages are effective at the point
    /// they are received in the stream; their timestamps are ignored.
    /// The client or the server sends this message to notify the peer about
    /// the user control events.
    #[deku(id = "4")]
    UserControl(Event),

    /// The client or the server sends this message to send audio data to the
    /// peer. The message type value of 8 is reserved for audio messages.
    #[deku(id = "8")]
    Audio(#[deku(count = "payload_length")] Vec<u8>),

    // The client or the server sends this message to send video data to the
    // peer. The message type value of 9 is reserved for video messages.
    #[deku(id = "9")]
    Video(#[deku(count = "payload_length")] Vec<u8>),

    /// The client or the server sends this message to send Metadata or any
    /// user data to the peer. Metadata includes details about the
    /// data(audio, video etc.) like creation time, duration, theme and so
    /// on. These messages have been assigned message type value of 18 for
    /// Amf0 and message type value of 15 for Amf3.
    #[deku(id_pat = "15 | 18")]
    Data(#[deku(ctx = "message_type, payload_length")] AmfValue),

    /// A shared object is a Flash object (a collection of name value pairs)
    /// that are in synchronization across multiple clients, instances, and
    /// so on. The message types 19 for Amf0 and 16 for Amf3 are reserved
    /// for shared object events. Each message can contain multiple events.
    ///
    /// ```no_rust
    /// +------+------+-------+-----+-----+------+-----+ +-----+------+-----+
    /// |Header|Shared|Current|Flags|Event|Event |Event|.|Event|Event |Event|
    /// |      |Object|Version|     |Type |data  |data |.|Type |data  |data |
    /// |      |Name |        |     |     |length|     |.|     |length|     |
    /// +------+------+-------+-----+-----+------+-----+ +-----+------+-----+
    ///        |                                                            |
    ///        |<- - - - - - - - - - - - - - - - - - - - - - - - - - - - - >|
    ///        |              AMF Shared Object Message body                |
    /// ```
    ///
    /// For information about the shared object event messages supported, see
    /// [section 7.1.3].
    ///
    /// [section 7.1.3]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=24
    #[deku(id_pat = "16 | 19")]
    SharedObject(#[deku(ctx = "message_type, payload_length")] AmfValue),

    /// Command messages carry the AMF-encoded commands between the client
    /// and the server. These messages have been assigned message type value
    /// of 20 for Amf0 encoding and message type value of 17 for Amf3
    /// encoding. These messages are sent to perform some operations like
    /// connect, createStream, publish, play, pause on the peer. Command
    /// messages like onstatus, result etc. are used to inform the sender
    /// about the status of the requested commands. A command message
    /// consists of command name, transaction ID, and command object that
    /// contains related parameters. A client or a server can request Remote
    /// Procedure Calls (RPC) over streams that are communicated using the
    /// command messages to the peer.
    #[deku(id_pat = "17 | 20")]
    Command(#[deku(ctx = "message_type, payload_length")] AmfValue),

    /// An aggregate message is a single message that contains a series of
    /// RTMP sub-messages using the format described in [Section 6.1]. Message
    /// type 22 is used for aggregate messages.
    ///
    /// ```no_rust
    /// +---------+-------------------------+
    /// | Header  | Aggregate Message body  |
    /// +---------+-------------------------+
    ///      The Aggregate Message format
    ///
    /// +--------+-------+---------+--------+-------+---------+ - - - -
    /// |Header 0|Message|Back     |Header 1|Message|Back     |
    /// |        |Data 0 |Pointer 0|        |Data 1 |Pointer 1|
    /// +--------+-------+---------+--------+-------+---------+ - - - -
    ///       The Aggregate Message body format
    /// ```
    ///
    /// The back pointer contains the size of the previous message including
    /// its header. It is included to match the format of FLV file and is
    /// used for backward seek.
    ///
    /// Using aggregate messages has several performance benefits:
    /// - The chunk stream can send at most a single complete message within
    ///   a chunk. Therefore, increasing the chunk size and using the aggregate
    //    message reduces the number of chunks sent.
    /// - The sub-messages can be stored contiguously in memory. It is more
    ///   efficient when making system calls to send the data on the network.
    ///
    /// [Section 6.1]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=22
    #[deku(id = "22")]
    Aggregate(#[deku(ctx = "payload_length, message_stream_id, message_timestamp")] Aggregate),
}
