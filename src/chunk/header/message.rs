use deku::prelude::*;

/// A message header, as described in [section 5.3.1.2] of the spec.
///
/// Message headers contain metadata about the message that's being communicated
/// along the chunkstream. While the data in a chunk is opaque to the chunk
/// stream protocol, this metadata is useful for parsing the chunk's payload
/// into messages.
///
/// In the spec, these are also called "chunk headers" or "chunk message
/// headers", but since they don't actually contain any metadata about the chunk
/// itself, we refer to them as message headers to avoid confusion.
///
/// [section 5.3.1.2]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=13
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(
    ctx = "message_header_type: u8",
    id = "message_header_type",
    endian = "big"
)]
pub enum MessageHeader {
    /// A type 0 message header, as described in [section 5.3.1.2.1] of the
    /// spec.
    /// Type 0 message headers are the longest, at 11 bytes, and are used at the
    /// start of a stream or whenever the stream timestamp goes backwards.
    ///
    /// [section 5.3.1.2.1]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=14
    #[deku(id = "0")]
    Type0 {
        /// The timestamp, relative to the epoch established during the handshake, of the current message.
        #[deku(bytes = 3)]
        timestamp: u32,

        /// The length of the message, in bytes -- _NOT_ the length of the current chunk.
        /// The message may span multiple chunks, or take up less than a single chunk.
        #[deku(bytes = 3)]
        message_length: u32,

        /// The type of the message. See [section 7.1 of the spec] for more
        ///
        /// [section 7.1 of the spec]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=24
        message_type_id: u8,

        /// Which message stream this message belongs to. One chunk stream multiplexes
        /// multiple message streams, so this field helps in de-multiplexing.
        #[deku(endian = "little")]
        message_stream_id: u32,
    },

    /// A type 1 message header, as described in [section 5.3.1.2.2] of the spec.
    /// Type 1 message headers are the second longest, at 7 bytes, and are used
    /// for each new message in a stream after the first when the underlying
    /// messages are of variable size.
    ///
    /// For type 1 (and all further) message headers, any
    /// omitted values take the same values as the preceding message. In the
    /// case of type 1 headers, for example, this means the message stream ID.
    ///
    /// [section 5.3.1.2.2]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=14
    #[deku(id = "1")]
    Type1 {
        /// The difference (in milliseconds) between the timestamp of this chunk
        /// and the timestamp of the previous chunk
        #[deku(bytes = 3)]
        timestamp_delta: u32,

        /// The length of the message, in bytes -- _NOT_ the length of the current chunk.
        /// The message may span multiple chunks, or take up less than a single chunk.
        #[deku(bytes = 3)]
        message_length: u32,

        /// The type of the message. See [section 7.1 of the spec] for more
        ///
        /// [section 7.1 of the spec]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=24
        message_type_id: u8,
    },

    /// A type 2 message header, as described in [section 5.3.1.2.3] of the spec.
    /// Type 2 message headers are 3 bytes long
    /// and contain only the timestamp delta. All other values are the same as the preceding message.
    /// These are used for every message
    /// after the first in a message stream with fixed-size messages.
    ///
    /// [section 5.3.1.2.3]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=15
    #[deku(id = "2")]
    Type2 {
        /// The difference (in milliseconds) between the timestamp of this chunk
        /// and the timestamp of the previous chunk
        #[deku(bytes = 3)]
        timestamp_delta: u32,
    },

    /// A type 3 message header, as described in [section 5.3.1.2.4] of the spec.
    /// Type 3 chunks don't actually have a message header. These are used for
    /// chunks which contain parts of a message, because another chunk has
    /// already communicated the message's metadata. All values are the same as those
    /// in the preceding chunk.
    ///
    /// [section 5.3.1.2.4]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=15
    #[deku(id = "3")]
    Type3,
}

impl MessageHeader {
    /// If the timestamp (delta) doesn't fit in 3 bytes, then the timestamp (delta) field
    /// is set to 0xFFFFFF, and a 32-bit "extended timestamp" is sent separately.
    /// See [section 5.3.1.3] of the spec for more.
    ///
    /// [section 5.3.1.3]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=16
    pub const fn has_extended_timestamp(&self) -> bool {
        match self {
            MessageHeader::Type0 { timestamp, .. } => *timestamp == 0xFFFFFF,
            MessageHeader::Type1 {
                timestamp_delta, ..
            } => *timestamp_delta == 0xFFFFFF,
            MessageHeader::Type2 { timestamp_delta } => *timestamp_delta == 0xFFFFFF,
            MessageHeader::Type3 => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::MessageHeader;
    use deku::{
        bitvec::{BitSlice, Msb0},
        prelude::*,
    };

    const TIMESTAMP_BYTES_1: u8 = 53;
    const TIMESTAMP_BYTES_2: u8 = 76;
    const TIMESTAMP_BYTES_3: u8 = 24;

    const TIMESTAMP: u32 =
        u32::from_be_bytes([0, TIMESTAMP_BYTES_1, TIMESTAMP_BYTES_2, TIMESTAMP_BYTES_3]);

    const MESSAGE_LENGTH_BYTES_1: u8 = 14;
    const MESSAGE_LENGTH_BYTES_2: u8 = 86;
    const MESSAGE_LENGTH_BYTES_3: u8 = 33;

    const MESSAGE_LENGTH: u32 = u32::from_be_bytes([
        0,
        MESSAGE_LENGTH_BYTES_1,
        MESSAGE_LENGTH_BYTES_2,
        MESSAGE_LENGTH_BYTES_3,
    ]);

    const MESSAGE_TYPE_ID: u8 = 13;

    const MESSAGE_STREAM_ID_BYTES_1: u8 = 24;
    const MESSAGE_STREAM_ID_BYTES_2: u8 = 1;
    const MESSAGE_STREAM_ID_BYTES_3: u8 = 96;
    const MESSAGE_STREAM_ID_BYTES_4: u8 = 100;

    const MESSAGE_STREAM_ID: u32 = u32::from_le_bytes([
        MESSAGE_STREAM_ID_BYTES_1,
        MESSAGE_STREAM_ID_BYTES_2,
        MESSAGE_STREAM_ID_BYTES_3,
        MESSAGE_STREAM_ID_BYTES_4,
    ]);

    #[cfg(test)]
    mod read {
        use super::*;

        #[test]
        fn type_zero() {
            const RAW_BYTES: [u8; 11] = [
                TIMESTAMP_BYTES_1,
                TIMESTAMP_BYTES_2,
                TIMESTAMP_BYTES_3,
                MESSAGE_LENGTH_BYTES_1,
                MESSAGE_LENGTH_BYTES_2,
                MESSAGE_LENGTH_BYTES_3,
                MESSAGE_TYPE_ID,
                MESSAGE_STREAM_ID_BYTES_1,
                MESSAGE_STREAM_ID_BYTES_2,
                MESSAGE_STREAM_ID_BYTES_3,
                MESSAGE_STREAM_ID_BYTES_4,
            ];

            let expected = MessageHeader::Type0 {
                timestamp: TIMESTAMP,
                message_length: MESSAGE_LENGTH,
                message_type_id: MESSAGE_TYPE_ID,
                message_stream_id: MESSAGE_STREAM_ID,
            };

            let input = BitSlice::<Msb0, u8>::from_slice(&RAW_BYTES).unwrap();
            let (_, actual) = MessageHeader::read(input, 0).unwrap();

            assert_eq!(actual, expected)
        }

        #[test]
        fn type_one() {
            const RAW_BYTES: [u8; 7] = [
                TIMESTAMP_BYTES_1,
                TIMESTAMP_BYTES_2,
                TIMESTAMP_BYTES_3,
                MESSAGE_LENGTH_BYTES_1,
                MESSAGE_LENGTH_BYTES_2,
                MESSAGE_LENGTH_BYTES_3,
                MESSAGE_TYPE_ID,
            ];

            let expected = MessageHeader::Type1 {
                timestamp_delta: TIMESTAMP,
                message_length: MESSAGE_LENGTH,
                message_type_id: MESSAGE_TYPE_ID,
            };

            let input = BitSlice::<Msb0, u8>::from_slice(&RAW_BYTES).unwrap();
            let (_, actual) = MessageHeader::read(input, 1).unwrap();

            assert_eq!(actual, expected)
        }

        #[test]
        fn type_two() {
            const RAW_BYTES: [u8; 3] = [TIMESTAMP_BYTES_1, TIMESTAMP_BYTES_2, TIMESTAMP_BYTES_3];

            let expected = MessageHeader::Type2 {
                timestamp_delta: TIMESTAMP,
            };

            let input = BitSlice::<Msb0, u8>::from_slice(&RAW_BYTES).unwrap();
            let (_, actual) = MessageHeader::read(input, 2).unwrap();

            assert_eq!(actual, expected)
        }

        #[test]
        fn type_three() {
            const RAW_BYTES: [u8; 0] = [];

            let expected = MessageHeader::Type3;

            let input = BitSlice::<Msb0, u8>::from_slice(&RAW_BYTES).unwrap();
            let (_, actual) = MessageHeader::read(input, 3).unwrap();

            assert_eq!(actual, expected)
        }
    }

    #[cfg(test)]
    mod write {}
}
