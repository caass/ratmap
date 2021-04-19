use deku::{
    bitvec::{bits, BitSlice, BitVec, Msb0},
    prelude::*,
};

#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct BasicHeader {
    #[deku(bits = 2)]
    pub message_header_type: u8,
    pub chunk_stream_id: ChunkStreamId,
}

#[derive(Debug, PartialEq)]
pub struct ChunkStreamId(pub u32);

/// Implementation taken from [the docs](https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=13).
impl<'a> DekuRead<'a> for ChunkStreamId {
    fn read(
        input: &'a BitSlice<Msb0, u8>,
        _ctx: (),
    ) -> Result<(&'a BitSlice<Msb0, u8>, Self), DekuError>
    where
        Self: Sized,
    {
        match input.leading_zeros() {
            // 2-byte header
            6 => {
                let (rest, minus_64) = u8::read(&input[6..], ())?;
                let chunk_stream_id = (minus_64 as u32) + 64;

                Ok((rest, ChunkStreamId(chunk_stream_id)))
            }
            // 3-byte header
            5 => {
                let (third_byte_input, second_byte) = u8::read(&input[6..], ())?;
                let (rest, third_byte) = u8::read(third_byte_input, ())?;

                let chunk_stream_id = (third_byte as u32 * 256) + second_byte as u32 + 64;
                Ok((rest, ChunkStreamId(chunk_stream_id)))
            }
            // 1-byte header
            _ => {
                // reading a u8 means we need 8 bits
                // rip zero-copy (not that i even know what that means)
                let padded = bits![mut Msb0, u8; 0, 0, 0, 0, 0, 0, 0, 0];
                padded[2..].copy_from_bitslice(&input[..6]);

                let (_, chunk_stream_id) = u8::read(&padded, ())?;
                Ok((&input[6..], ChunkStreamId(chunk_stream_id as u32)))
            }
        }
    }
}

// Attempt at inverting DekuRead...I wonder if there's a way to provide a blanket DekuWrite for anything that impl's DekuRead? Maybe...
impl DekuWrite for ChunkStreamId {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _ctx: ()) -> Result<(), DekuError> {
        let bytes = match self.0 {
            // 1-byte header fits in 6 bits
            2..=63 => {
                let [.., byte] = self.0.to_be_bytes();

                Ok(vec![byte])
            }

            // 2-byte header maxes out at 255 (8 bits) + 64
            64..=319 => {
                let [.., byte] = (self.0 - 64).to_be_bytes();
                Ok(vec![0, byte])
            }

            // 3-byte header maxes out at second byte max + (255 * 256) = 65599
            320..=65599 => {
                let remainder = self.0 % 256;
                let third_byte = ((self.0 - remainder) / 256) as u8;
                let second_byte = (remainder - 64) as u8;

                Ok(vec![1, second_byte, third_byte])
            }
            0 | 1 => Err(DekuError::InvalidParam(format!(
                "Chunk Stream ID's 0 & 1 are reserved! Attempted to write ID: {}",
                self.0
            ))),
            _ => Err(DekuError::InvalidParam(format!(
                "RTMP only supports Chunk Stream ID's up to 65599! Attempted to write ID: {}",
                self.0
            ))),
        }?;

        if let Ok(bits) = BitSlice::<Msb0, u8>::from_slice(bytes.as_slice()) {
            // skip first 2 bits
            output.extend_from_bitslice(&bits[2..]);
            Ok(())
        } else {
            Err(DekuError::Parse(format!(
                "Failed to parse {:#?} to BitSlice!",
                bytes
            )))
        }
    }
}

#[cfg(test)]
mod test {
    use super::{BasicHeader, ChunkStreamId};
    use deku::prelude::*;

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

    #[cfg(test)]
    mod read {
        use super::*;

        #[cfg(test)]
        mod message_header_type {
            use super::*;

            #[test]
            fn type_zero() {
                let input = [MESSAGE_HEADER_TYPE_ZERO | ONE_BYTE_CSID];
                let expected = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn type_one() {
                let input = [MESSAGE_HEADER_TYPE_ONE | ONE_BYTE_CSID];
                let expected = BasicHeader {
                    message_header_type: 1,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn type_two() {
                let input = [MESSAGE_HEADER_TYPE_TWO | ONE_BYTE_CSID];
                let expected = BasicHeader {
                    message_header_type: 2,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn type_three() {
                let input = [MESSAGE_HEADER_TYPE_THREE | ONE_BYTE_CSID];

                let expected = BasicHeader {
                    message_header_type: 3,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();

                assert_eq!(expected, actual);
            }
        }

        #[cfg(test)]
        mod chunk_stream_id {
            use super::*;

            #[test]
            fn one_byte() {
                let input = [MESSAGE_HEADER_TYPE_ZERO | ONE_BYTE_CSID];
                let expected = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn two_byte() {
                let input = [
                    MESSAGE_HEADER_TYPE_ZERO | TWO_BYTE_CSID_MARKER,
                    TWO_BYTE_CSID,
                ];
                let expected = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId((TWO_BYTE_CSID as u32) + 64),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn three_byte() {
                let input = [
                    MESSAGE_HEADER_TYPE_ZERO | THREE_BYTE_CSID_MARKER,
                    THREE_BYTE_CSID_1,
                    THREE_BYTE_CSID_2,
                ];
                let expected = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(
                        (THREE_BYTE_CSID_1 as u32) + 64 + ((THREE_BYTE_CSID_2 as u32) * 256),
                    ),
                };

                let (_, actual) = BasicHeader::from_bytes((&input, 0)).unwrap();

                assert_eq!(expected, actual);
            }
        }
    }

    #[cfg(test)]
    mod write {
        use super::*;

        #[cfg(test)]
        mod message_header_type {
            use super::*;

            #[test]
            fn type_zero() {
                let input = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let expected = vec![MESSAGE_HEADER_TYPE_ZERO | ONE_BYTE_CSID];
                let actual = input.to_bytes().unwrap();

                assert_eq!(expected, actual);
            }

            #[test]
            fn type_one() {
                let input = BasicHeader {
                    message_header_type: 1,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let expected = vec![MESSAGE_HEADER_TYPE_ONE | ONE_BYTE_CSID];
                let actual = input.to_bytes().unwrap();

                assert_eq!(expected, actual);
            }

            #[test]
            fn type_two() {
                let input = BasicHeader {
                    message_header_type: 2,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let expected = vec![MESSAGE_HEADER_TYPE_TWO | ONE_BYTE_CSID];
                let actual = input.to_bytes().unwrap();

                assert_eq!(expected, actual);
            }

            #[test]
            fn type_three() {
                let input = BasicHeader {
                    message_header_type: 3,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };

                let expected = vec![MESSAGE_HEADER_TYPE_THREE | ONE_BYTE_CSID];
                let actual = input.to_bytes().unwrap();

                assert_eq!(expected, actual);
            }
        }

        #[cfg(test)]
        mod chunk_stream_id {
            use super::*;

            #[test]
            fn one_byte() {
                let input = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(ONE_BYTE_CSID as u32),
                };
                let expected = vec![MESSAGE_HEADER_TYPE_ZERO | ONE_BYTE_CSID];

                let actual = input.to_bytes().unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn two_byte() {
                let input = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(TWO_BYTE_CSID as u32 + 64),
                };
                let expected = vec![
                    MESSAGE_HEADER_TYPE_ZERO | TWO_BYTE_CSID_MARKER,
                    TWO_BYTE_CSID,
                ];

                let actual = input.to_bytes().unwrap();
                assert_eq!(expected, actual);
            }

            #[test]
            fn three_byte() {
                let input = BasicHeader {
                    message_header_type: 0,
                    chunk_stream_id: ChunkStreamId(
                        (THREE_BYTE_CSID_1 as u32 + 64) + (THREE_BYTE_CSID_2 as u32 * 256),
                    ),
                };
                let expected = vec![
                    MESSAGE_HEADER_TYPE_ZERO | THREE_BYTE_CSID_MARKER,
                    THREE_BYTE_CSID_1,
                    THREE_BYTE_CSID_2,
                ];

                let actual = input.to_bytes().unwrap();
                assert_eq!(expected, actual);
            }
        }
    }
}
