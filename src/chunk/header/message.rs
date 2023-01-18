//! 5.3.1.2. [Chunk Message Header](https://rtmp.veriskope.com/docs/spec/#5312-chunk-message-header)

use deku::prelude::*;

/// This field encodes information about the message being sent (whether in whole or in part).
/// The length can be determined using the chunk type specified in the chunk header.
#[derive(PartialEq, Debug, DekuRead, DekuWrite)]
#[deku(ctx = "chunk_type: u8", id = "chunk_type", endian = "big")]
pub enum MessageHeader {
    /// [5.3.1.2.1. Type 0](https://rtmp.veriskope.com/docs/spec/#53121type-0)
    ///
    /// Type 0 chunk headers are 11 bytes long.
    ///
    /// This type MUST be used at the start of a chunk stream,
    /// and whenever the stream timestamp goes backward
    /// (e.g., because of a backward seek).
    #[deku(id = "0")]
    BeginOrRewindStream {
        /// For a type-0 chunk, the absolute timestamp of the message is sent here.
        /// If the timestamp is greater than or equal to 16777215 (hexadecimal 0xFFFFFF),
        /// this field MUST be 16777215, indicating the presence of the Extended Timestamp
        /// field to encode the full 32 bit timestamp.
        /// Otherwise, this field SHOULD be the entire timestamp.
        #[deku(bytes = "3")]
        timestamp: u32,

        /// For a type-0 or type-1 chunk, the length of the message is sent here.
        /// Note that this is generally not the same as the length of the chunk payload.
        /// The chunk payload length is the maximum chunk size for all but the last chunk,
        /// and the remainder (which may be the entire length, for small messages) for the last chunk.
        #[deku(bytes = "3")]
        message_length: u32,

        /// For a type-0 or type-1 chunk, type of the message is sent here.
        message_type_id: u8,

        /// For a type-0 chunk, the message stream ID is stored.
        /// Message stream ID is stored in little-endian format.
        ///
        /// Typically, all messages in the same chunk stream will come from the same message stream.
        /// While it is possible to multiplex separate message streams into the same chunk stream,
        /// this defeats the benefits of the header compression.
        ///
        /// However, if one message stream is closed and another one subsequently opened,
        /// there is no reason an existing chunk stream cannot be reused by sending a new type-0 chunk.
        #[deku(endian = "little")]
        message_stream_id: u32,
    },

    /// [5.3.1.2.2. Type 1](https://rtmp.veriskope.com/docs/spec/#53122-type-1)
    ///
    /// Type 1 chunk headers are 7 bytes long.
    ///
    /// The message stream ID is not included; this chunk takes the same stream ID as the preceding chunk.
    /// Streams with variable-sized messages (for example, many video formats)
    /// SHOULD use this format for the first chunk of each new message after the first.
    #[deku(id = "1")]
    BeginVariableLengthMessage {
        /// For a type-1 or type-2 chunk, the difference between the previous chunk’s timestamp
        /// and the current chunk’s timestamp is sent here.
        /// If the delta is greater than or equal to 16777215 (hexadecimal 0xFFFFFF),
        /// this field MUST be 16777215, indicating the presence of the Extended Timestamp
        /// field to encode the full 32 bit delta. Otherwise, this field SHOULD be the actual delta.
        #[deku(bytes = "3")]
        timestamp_delta: u32,

        /// For a type-0 or type-1 chunk, the length of the message is sent here.
        /// Note that this is generally not the same as the length of the chunk payload.
        /// The chunk payload length is the maximum chunk size for all but the last chunk,
        /// and the remainder (which may be the entire length, for small messages) for the last chunk.
        #[deku(bytes = "3")]
        message_length: u32,

        /// For a type-0 or type-1 chunk, type of the message is sent here.
        message_type_id: u8,
    },

    /// [5.3.1.2.3. Type 2](https://rtmp.veriskope.com/docs/spec/#53123-type-2)
    ///
    /// Type 2 chunk headers are 3 bytes long.
    ///
    /// Neither the stream ID nor the message length is included;
    /// this chunk has the same stream ID and message length as the preceding chunk.
    /// Streams with constant-sized messages (for example, some audio and data formats)
    /// SHOULD use this format for the first chunk of each message after the first.
    #[deku(id = "2")]
    BeginConstantLengthMessage {
        /// For a type-1 or type-2 chunk, the difference between the previous chunk’s timestamp
        /// and the current chunk’s timestamp is sent here.
        /// If the delta is greater than or equal to 16777215 (hexadecimal 0xFFFFFF),
        /// this field MUST be 16777215, indicating the presence of the Extended Timestamp
        /// field to encode the full 32 bit delta. Otherwise, this field SHOULD be the actual delta.
        #[deku(bytes = "3")]
        timestamp_delta: u32,
    },

    /// [5.3.1.2.4. Type 3](https://rtmp.veriskope.com/docs/spec/#53124-type-3)
    /// Type 3 chunks have no message header.
    ///
    /// The stream ID, message length and timestamp delta fields are not present;
    /// chunks of this type take values from the preceding chunk for the same Chunk Stream ID.
    ///
    /// When a single message is split into chunks,
    /// all chunks of a message except the first one SHOULD use this type.
    /// Refer to Example 2 ([Section 5.3.2.2](https://rtmp.veriskope.com/docs/spec/#5322-example-2)).
    ///
    /// A stream consisting of messages of exactly the same size,
    /// stream ID and spacing in time SHOULD use this type for all chunks after a chunk of Type 2.
    /// Refer to Example 1 ([Section 5.3.2.1](https://rtmp.veriskope.com/docs/spec/#5321-example-1)).
    ///
    /// If the delta between the first message and the second message is same as the timestamp
    /// of the first message, then a chunk of Type 3 could immediately follow the chunk of Type 0
    /// as there is no need for a chunk of Type 2 to register the delta.
    ///
    /// If a Type 3 chunk follows a Type 0 chunk, then the timestamp delta for this Type 3 chunk
    /// is the same as the timestamp of the Type 0 chunk.
    #[deku(id = "3")]
    ContinueMessage,
}

impl MessageHeader {
    pub fn size(&self) -> usize {
        match self {
            Self::BeginOrRewindStream { .. } => 11,
            Self::BeginVariableLengthMessage { .. } => 7,
            Self::BeginConstantLengthMessage { .. } => 3,
            Self::ContinueMessage => 0,
        }
    }

    pub fn has_extended_timestamp(&self, last_timestamp: u32) -> bool {
        match *self {
            MessageHeader::BeginOrRewindStream { timestamp, .. } => timestamp == 0xFFFFFF,
            MessageHeader::BeginVariableLengthMessage {
                timestamp_delta, ..
            }
            | MessageHeader::BeginConstantLengthMessage { timestamp_delta } => {
                timestamp_delta == 0xFFFFFF
            }
            MessageHeader::ContinueMessage => last_timestamp >= 0xFFFFFF,
        }
    }
}
