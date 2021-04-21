mod basic;
mod message;

use basic::BasicHeader;
use deku::prelude::*;
use message::MessageHeader;

/// Each chunk consists of a header and data. The header itself has
/// three parts: the basic header, the message header, and an optional extended timestamp.
/// Chunk headers are between 1 and 18 bytes
///
/// ````no_rust
///  +--------------+----------------+--------------------+--------------+
///  | Basic Header | Message Header | Extended Timestamp | Chunk Data |
///  +--------------+----------------+--------------------+--------------+
///  |                                                    |
///  |<------------------- Chunk Header ----------------->|
/// ```
///
/// See [section 5.3.1] for more details.
///
/// [section 5.3.1]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=11
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct ChunkHeader {
    pub basic_header: BasicHeader,

    #[deku(ctx = "basic_header.message_header_type")]
    pub message_header: MessageHeader,

    #[deku(endian = "big", cond = "message_header.has_extended_timestamp()")]
    pub extended_timestamp: Option<u32>,
}

#[cfg(test)]
mod test {
    use super::{
        basic::{BasicHeader, ChunkStreamId},
        message::MessageHeader,
        ChunkHeader,
    };
    use deku::{
        bitvec::{BitSlice, Msb0},
        prelude::*,
    };

    const MESSAGE_HEADER_TYPE_ZERO: u8 = 0b00000000;
    const MESSAGE_HEADER_TYPE_ONE: u8 = 0b01000000;
    const MESSAGE_HEADER_TYPE_TWO: u8 = 0b10000000;
    const MESSAGE_HEADER_TYPE_THREE: u8 = 0b11000000;

    const TWO_BYTE_CSID_MARKER: u8 = 0b00000000;
    const THREE_BYTE_CSID_MARKER: u8 = 0b00000001;

    const ONE_BYTE_CSID: u8 = 0b00110111;
    const TWO_BYTE_CSID: u8 = 0b10101011;
    const THREE_BYTE_CSID_1: u8 = 0b10101010;
    const THREE_BYTE_CSID_2: u8 = 0b11011010;

    const TIMESTAMP_BYTES_1: u8 = 53;
    const TIMESTAMP_BYTES_2: u8 = 76;
    const TIMESTAMP_BYTES_3: u8 = 24;

    const TIMESTAMP: u32 =
        u32::from_be_bytes([0, TIMESTAMP_BYTES_1, TIMESTAMP_BYTES_2, TIMESTAMP_BYTES_3]);

    const EXTENDED_TIMESTAMP_MARKER_BYTE: u8 = 0xFF;

    const EXTENDED_TIMESTAMP_MARKER: u32 = u32::from_be_bytes([
        0,
        EXTENDED_TIMESTAMP_MARKER_BYTE,
        EXTENDED_TIMESTAMP_MARKER_BYTE,
        EXTENDED_TIMESTAMP_MARKER_BYTE,
    ]);

    const EXTENDED_TIMESTAMP_BYTE_1: u8 = 0xAB;
    const EXTENDED_TIMESTAMP_BYTE_2: u8 = 0xFF;
    const EXTENDED_TIMESTAMP_BYTE_3: u8 = 0xFF;
    const EXTENDED_TIMESTAMP_BYTE_4: u8 = 0xFF;

    const EXTENDED_TIMESTAMP: u32 = u32::from_be_bytes([
        EXTENDED_TIMESTAMP_BYTE_1,
        EXTENDED_TIMESTAMP_BYTE_2,
        EXTENDED_TIMESTAMP_BYTE_3,
        EXTENDED_TIMESTAMP_BYTE_4,
    ]);

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
        fn basic_and_message_header() {
            for &(message_header_type_byte, message_header_type) in [
                (MESSAGE_HEADER_TYPE_ZERO, 0u8),
                (MESSAGE_HEADER_TYPE_ONE, 1u8),
                (MESSAGE_HEADER_TYPE_TWO, 2u8),
                (MESSAGE_HEADER_TYPE_THREE, 3u8),
            ]
            .iter()
            {
                for &chunk_stream_id in [
                    u16::from_be_bytes([0, ONE_BYTE_CSID]),
                    u16::from_be_bytes([0, TWO_BYTE_CSID]),
                    u16::from_be_bytes([THREE_BYTE_CSID_1, THREE_BYTE_CSID_2]),
                ]
                .iter()
                {
                    let (basic_header_bytes, basic_header_expected) = {
                        let [.., byte] = chunk_stream_id.to_be_bytes();
                        match byte {
                            ONE_BYTE_CSID => (
                                vec![message_header_type_byte | ONE_BYTE_CSID],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                                },
                            ),
                            TWO_BYTE_CSID => (
                                vec![
                                    message_header_type_byte | TWO_BYTE_CSID_MARKER,
                                    TWO_BYTE_CSID,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(TWO_BYTE_CSID as u32 + 64),
                                },
                            ),
                            THREE_BYTE_CSID_2 => (
                                vec![
                                    message_header_type_byte | THREE_BYTE_CSID_MARKER,
                                    THREE_BYTE_CSID_1,
                                    THREE_BYTE_CSID_2,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(
                                        THREE_BYTE_CSID_1 as u32
                                            + 64
                                            + THREE_BYTE_CSID_2 as u32 * 256,
                                    ),
                                },
                            ),
                            _ => unreachable!(),
                        }
                    };

                    let (message_header_bytes, message_header_expected) =
                        match message_header_type_byte {
                            MESSAGE_HEADER_TYPE_ZERO => (
                                vec![
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
                                ],
                                MessageHeader::Type0 {
                                    timestamp: TIMESTAMP,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                    message_stream_id: MESSAGE_STREAM_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_ONE => (
                                vec![
                                    TIMESTAMP_BYTES_1,
                                    TIMESTAMP_BYTES_2,
                                    TIMESTAMP_BYTES_3,
                                    MESSAGE_LENGTH_BYTES_1,
                                    MESSAGE_LENGTH_BYTES_2,
                                    MESSAGE_LENGTH_BYTES_3,
                                    MESSAGE_TYPE_ID,
                                ],
                                MessageHeader::Type1 {
                                    timestamp_delta: TIMESTAMP,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_TWO => (
                                vec![TIMESTAMP_BYTES_1, TIMESTAMP_BYTES_2, TIMESTAMP_BYTES_3],
                                MessageHeader::Type2 {
                                    timestamp_delta: TIMESTAMP,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_THREE => (vec![], MessageHeader::Type3),
                            _ => unreachable!(),
                        };

                    let bytes = [&basic_header_bytes[..], &message_header_bytes[..]].concat();
                    let expected = ChunkHeader {
                        basic_header: basic_header_expected,
                        message_header: message_header_expected,
                        extended_timestamp: None,
                    };

                    let input = BitSlice::<Msb0, u8>::from_slice(bytes.as_slice()).unwrap();

                    let (_, actual) = ChunkHeader::read(input, ()).unwrap();
                    assert_eq!(expected, actual);
                }
            }
        }

        #[test]
        fn extended_timestamp() {
            for &(message_header_type_byte, message_header_type) in [
                (MESSAGE_HEADER_TYPE_ZERO, 0u8),
                (MESSAGE_HEADER_TYPE_ONE, 1u8),
                (MESSAGE_HEADER_TYPE_TWO, 2u8),
                (MESSAGE_HEADER_TYPE_THREE, 3u8),
            ]
            .iter()
            {
                for &chunk_stream_id in [
                    u16::from_be_bytes([0, ONE_BYTE_CSID]),
                    u16::from_be_bytes([0, TWO_BYTE_CSID]),
                    u16::from_be_bytes([THREE_BYTE_CSID_1, THREE_BYTE_CSID_2]),
                ]
                .iter()
                {
                    let (basic_header_bytes, basic_header_expected) = {
                        let [.., byte] = chunk_stream_id.to_be_bytes();
                        match byte {
                            ONE_BYTE_CSID => (
                                vec![message_header_type_byte | ONE_BYTE_CSID],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                                },
                            ),
                            TWO_BYTE_CSID => (
                                vec![
                                    message_header_type_byte | TWO_BYTE_CSID_MARKER,
                                    TWO_BYTE_CSID,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(TWO_BYTE_CSID as u32 + 64),
                                },
                            ),
                            THREE_BYTE_CSID_2 => (
                                vec![
                                    message_header_type_byte | THREE_BYTE_CSID_MARKER,
                                    THREE_BYTE_CSID_1,
                                    THREE_BYTE_CSID_2,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(
                                        THREE_BYTE_CSID_1 as u32
                                            + 64
                                            + THREE_BYTE_CSID_2 as u32 * 256,
                                    ),
                                },
                            ),
                            _ => unreachable!(),
                        }
                    };

                    let (message_header_bytes, message_header_expected) =
                        match message_header_type_byte {
                            MESSAGE_HEADER_TYPE_ZERO => (
                                vec![
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    MESSAGE_LENGTH_BYTES_1,
                                    MESSAGE_LENGTH_BYTES_2,
                                    MESSAGE_LENGTH_BYTES_3,
                                    MESSAGE_TYPE_ID,
                                    MESSAGE_STREAM_ID_BYTES_1,
                                    MESSAGE_STREAM_ID_BYTES_2,
                                    MESSAGE_STREAM_ID_BYTES_3,
                                    MESSAGE_STREAM_ID_BYTES_4,
                                    EXTENDED_TIMESTAMP_BYTE_1,
                                    EXTENDED_TIMESTAMP_BYTE_2,
                                    EXTENDED_TIMESTAMP_BYTE_3,
                                    EXTENDED_TIMESTAMP_BYTE_4,
                                ],
                                MessageHeader::Type0 {
                                    timestamp: EXTENDED_TIMESTAMP_MARKER,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                    message_stream_id: MESSAGE_STREAM_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_ONE => (
                                vec![
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    MESSAGE_LENGTH_BYTES_1,
                                    MESSAGE_LENGTH_BYTES_2,
                                    MESSAGE_LENGTH_BYTES_3,
                                    MESSAGE_TYPE_ID,
                                    EXTENDED_TIMESTAMP_BYTE_1,
                                    EXTENDED_TIMESTAMP_BYTE_2,
                                    EXTENDED_TIMESTAMP_BYTE_3,
                                    EXTENDED_TIMESTAMP_BYTE_4,
                                ],
                                MessageHeader::Type1 {
                                    timestamp_delta: EXTENDED_TIMESTAMP_MARKER,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_TWO => (
                                vec![
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_BYTE_1,
                                    EXTENDED_TIMESTAMP_BYTE_2,
                                    EXTENDED_TIMESTAMP_BYTE_3,
                                    EXTENDED_TIMESTAMP_BYTE_4,
                                ],
                                MessageHeader::Type2 {
                                    timestamp_delta: EXTENDED_TIMESTAMP_MARKER,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_THREE => (vec![], MessageHeader::Type3),
                            _ => unreachable!(),
                        };

                    let bytes = [&basic_header_bytes[..], &message_header_bytes[..]].concat();
                    let extended_timestamp_expected = match message_header_expected {
                        MessageHeader::Type3 => None,
                        _ => Some(EXTENDED_TIMESTAMP),
                    };
                    let expected = ChunkHeader {
                        basic_header: basic_header_expected,
                        message_header: message_header_expected,
                        extended_timestamp: extended_timestamp_expected,
                    };

                    let input = BitSlice::<Msb0, u8>::from_slice(bytes.as_slice()).unwrap();

                    let (_, actual) = ChunkHeader::read(input, ()).unwrap();
                    assert_eq!(expected, actual);
                }
            }
        }
    }

    #[cfg(test)]
    mod write {
        use super::*;

        #[test]
        fn basic_and_message_header() {
            for &(message_header_type_byte, message_header_type) in [
                (MESSAGE_HEADER_TYPE_ZERO, 0u8),
                (MESSAGE_HEADER_TYPE_ONE, 1u8),
                (MESSAGE_HEADER_TYPE_TWO, 2u8),
                (MESSAGE_HEADER_TYPE_THREE, 3u8),
            ]
            .iter()
            {
                for &chunk_stream_id in [
                    u16::from_be_bytes([0, ONE_BYTE_CSID]),
                    u16::from_be_bytes([0, TWO_BYTE_CSID]),
                    u16::from_be_bytes([THREE_BYTE_CSID_1, THREE_BYTE_CSID_2]),
                ]
                .iter()
                {
                    let (basic_header_expected, basic_header_input) = {
                        let [.., byte] = chunk_stream_id.to_be_bytes();
                        match byte {
                            ONE_BYTE_CSID => (
                                vec![message_header_type_byte | ONE_BYTE_CSID],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                                },
                            ),
                            TWO_BYTE_CSID => (
                                vec![
                                    message_header_type_byte | TWO_BYTE_CSID_MARKER,
                                    TWO_BYTE_CSID,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(TWO_BYTE_CSID as u32 + 64),
                                },
                            ),
                            THREE_BYTE_CSID_2 => (
                                vec![
                                    message_header_type_byte | THREE_BYTE_CSID_MARKER,
                                    THREE_BYTE_CSID_1,
                                    THREE_BYTE_CSID_2,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(
                                        THREE_BYTE_CSID_1 as u32
                                            + 64
                                            + THREE_BYTE_CSID_2 as u32 * 256,
                                    ),
                                },
                            ),
                            _ => unreachable!(),
                        }
                    };

                    let (message_header_expected, message_header_input) =
                        match message_header_type_byte {
                            MESSAGE_HEADER_TYPE_ZERO => (
                                vec![
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
                                ],
                                MessageHeader::Type0 {
                                    timestamp: TIMESTAMP,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                    message_stream_id: MESSAGE_STREAM_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_ONE => (
                                vec![
                                    TIMESTAMP_BYTES_1,
                                    TIMESTAMP_BYTES_2,
                                    TIMESTAMP_BYTES_3,
                                    MESSAGE_LENGTH_BYTES_1,
                                    MESSAGE_LENGTH_BYTES_2,
                                    MESSAGE_LENGTH_BYTES_3,
                                    MESSAGE_TYPE_ID,
                                ],
                                MessageHeader::Type1 {
                                    timestamp_delta: TIMESTAMP,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_TWO => (
                                vec![TIMESTAMP_BYTES_1, TIMESTAMP_BYTES_2, TIMESTAMP_BYTES_3],
                                MessageHeader::Type2 {
                                    timestamp_delta: TIMESTAMP,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_THREE => (vec![], MessageHeader::Type3),
                            _ => unreachable!(),
                        };

                    let expected =
                        [&basic_header_expected[..], &message_header_expected[..]].concat();
                    let input = ChunkHeader {
                        basic_header: basic_header_input,
                        message_header: message_header_input,
                        extended_timestamp: None,
                    };

                    let actual = input.to_bytes().unwrap();

                    assert_eq!(expected, actual);
                }
            }
        }

        #[test]
        fn extended_timestamp() {
            for &(message_header_type_byte, message_header_type) in [
                (MESSAGE_HEADER_TYPE_ZERO, 0u8),
                (MESSAGE_HEADER_TYPE_ONE, 1u8),
                (MESSAGE_HEADER_TYPE_TWO, 2u8),
                (MESSAGE_HEADER_TYPE_THREE, 3u8),
            ]
            .iter()
            {
                for &chunk_stream_id in [
                    u16::from_be_bytes([0, ONE_BYTE_CSID]),
                    u16::from_be_bytes([0, TWO_BYTE_CSID]),
                    u16::from_be_bytes([THREE_BYTE_CSID_1, THREE_BYTE_CSID_2]),
                ]
                .iter()
                {
                    let (basic_header_expected, basic_header_input) = {
                        let [.., byte] = chunk_stream_id.to_be_bytes();
                        match byte {
                            ONE_BYTE_CSID => (
                                vec![message_header_type_byte | ONE_BYTE_CSID],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                                },
                            ),
                            TWO_BYTE_CSID => (
                                vec![
                                    message_header_type_byte | TWO_BYTE_CSID_MARKER,
                                    TWO_BYTE_CSID,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(TWO_BYTE_CSID as u32 + 64),
                                },
                            ),
                            THREE_BYTE_CSID_2 => (
                                vec![
                                    message_header_type_byte | THREE_BYTE_CSID_MARKER,
                                    THREE_BYTE_CSID_1,
                                    THREE_BYTE_CSID_2,
                                ],
                                BasicHeader {
                                    message_header_type,
                                    chunk_stream_id: ChunkStreamId(
                                        THREE_BYTE_CSID_1 as u32
                                            + 64
                                            + THREE_BYTE_CSID_2 as u32 * 256,
                                    ),
                                },
                            ),
                            _ => unreachable!(),
                        }
                    };

                    let (message_header_expected, message_header_input) =
                        match message_header_type_byte {
                            MESSAGE_HEADER_TYPE_ZERO => (
                                vec![
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    MESSAGE_LENGTH_BYTES_1,
                                    MESSAGE_LENGTH_BYTES_2,
                                    MESSAGE_LENGTH_BYTES_3,
                                    MESSAGE_TYPE_ID,
                                    MESSAGE_STREAM_ID_BYTES_1,
                                    MESSAGE_STREAM_ID_BYTES_2,
                                    MESSAGE_STREAM_ID_BYTES_3,
                                    MESSAGE_STREAM_ID_BYTES_4,
                                    EXTENDED_TIMESTAMP_BYTE_1,
                                    EXTENDED_TIMESTAMP_BYTE_2,
                                    EXTENDED_TIMESTAMP_BYTE_3,
                                    EXTENDED_TIMESTAMP_BYTE_4,
                                ],
                                MessageHeader::Type0 {
                                    timestamp: EXTENDED_TIMESTAMP_MARKER,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                    message_stream_id: MESSAGE_STREAM_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_ONE => (
                                vec![
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    MESSAGE_LENGTH_BYTES_1,
                                    MESSAGE_LENGTH_BYTES_2,
                                    MESSAGE_LENGTH_BYTES_3,
                                    MESSAGE_TYPE_ID,
                                    EXTENDED_TIMESTAMP_BYTE_1,
                                    EXTENDED_TIMESTAMP_BYTE_2,
                                    EXTENDED_TIMESTAMP_BYTE_3,
                                    EXTENDED_TIMESTAMP_BYTE_4,
                                ],
                                MessageHeader::Type1 {
                                    timestamp_delta: EXTENDED_TIMESTAMP_MARKER,
                                    message_length: MESSAGE_LENGTH,
                                    message_type_id: MESSAGE_TYPE_ID,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_TWO => (
                                vec![
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_MARKER_BYTE,
                                    EXTENDED_TIMESTAMP_BYTE_1,
                                    EXTENDED_TIMESTAMP_BYTE_2,
                                    EXTENDED_TIMESTAMP_BYTE_3,
                                    EXTENDED_TIMESTAMP_BYTE_4,
                                ],
                                MessageHeader::Type2 {
                                    timestamp_delta: EXTENDED_TIMESTAMP_MARKER,
                                },
                            ),
                            MESSAGE_HEADER_TYPE_THREE => (vec![], MessageHeader::Type3),
                            _ => unreachable!(),
                        };

                    let expected =
                        [&basic_header_expected[..], &message_header_expected[..]].concat();
                    let extended_timestamp_input = match message_header_input {
                        MessageHeader::Type3 => None,
                        _ => Some(EXTENDED_TIMESTAMP),
                    };

                    let input = ChunkHeader {
                        basic_header: basic_header_input,
                        message_header: message_header_input,
                        extended_timestamp: extended_timestamp_input,
                    };

                    let actual = input.to_bytes().unwrap();

                    assert_eq!(expected, actual);
                }
            }
        }
    }
}
