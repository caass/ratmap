//! 5.3.1.1. [Chunk Basic Header](https://rtmp.veriskope.com/docs/spec/#5311-chunk-basic-header)

use std::convert::{TryFrom, TryInto};

use deku::prelude::*;
use thiserror::Error;

/// The Chunk Basic Header encodes the chunk stream ID and the chunk type. Chunk type determines the format of the encoded message header. Chunk Basic Header field may be 1, 2, or 3 bytes, depending on the chunk stream ID.
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
pub struct BasicHeader {
    /// Chunk type determines the format of the encoded message header
    #[deku(bits = "2")]
    pub(super) chunk_type: u8,
    chunk_stream_id: ChunkStreamId,
}

impl BasicHeader {
    pub fn begin_or_rewind_stream(chunk_stream_id: u32) -> Result<Self, ChunkStreamIdTryFromError> {
        Ok(Self {
            chunk_type: 0,
            chunk_stream_id: chunk_stream_id.try_into()?,
        })
    }

    pub fn begin_variable_length_message(
        chunk_stream_id: u32,
    ) -> Result<Self, ChunkStreamIdTryFromError> {
        Ok(Self {
            chunk_type: 1,
            chunk_stream_id: chunk_stream_id.try_into()?,
        })
    }

    pub fn begin_constant_length_message(
        chunk_stream_id: u32,
    ) -> Result<Self, ChunkStreamIdTryFromError> {
        Ok(Self {
            chunk_type: 2,
            chunk_stream_id: chunk_stream_id.try_into()?,
        })
    }

    pub fn continue_message(chunk_stream_id: u32) -> Result<Self, ChunkStreamIdTryFromError> {
        Ok(Self {
            chunk_type: 3,
            chunk_stream_id: chunk_stream_id.try_into()?,
        })
    }

    pub fn size(&self) -> usize {
        match self.chunk_stream_id {
            ChunkStreamId::OneByte(_) => 1,
            ChunkStreamId::TwoBytes(_) => 2,
            ChunkStreamId::ThreeBytes(_) => 3,
        }
    }

    pub fn chunk_stream_id(&self) -> u32 {
        self.chunk_stream_id.into()
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Copy)]
#[deku(type = "u8", bits = "6")]
#[deku(endian = "big")]
enum ChunkStreamId {
    /// Chunk stream IDs 2-63 can be encoded in the 1-byte version of this field.
    ///
    /// ```text
    ///  0 1 2 3 4 5 6 7
    /// +-+-+-+-+-+-+-+-+
    /// |fmt|   cs id   |
    /// +-+-+-+-+-+-+-+-+
    /// ```
    #[deku(id_pat = "0x2..=63")]
    OneByte(#[deku(bits = "6")] u8),

    /// Chunk stream IDs 64-319 can be encoded in the 2-byte form of the header. ID is computed as (the second byte + 64).
    ///
    /// ```text
    ///  0                   1
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |fmt|0 0 0 0 0 0|   cs id - 64  |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    #[deku(id = "0")]
    TwoBytes(u8),

    /// Chunk stream IDs 64-65599 can be encoded in the 3-byte version of this field. ID is computed as ((the third byte)*256 + (the second byte) + 64).
    /// ```text
    ///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// |fmt|0 0 0 0 0 1|          cs id - 64           |
    /// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
    /// ```
    #[deku(id = "1")]
    ThreeBytes(u16),
}

impl ChunkStreamId {
    pub const PROTOCOL_CONTROL: Self = Self::OneByte(2);
}

#[derive(Debug, Error)]
pub enum ChunkStreamIdTryFromError {
    #[error("Attempted to use reserved value as chunk stream ID.")]
    Reserved,
    #[error("Chunk stream ID exceeds maximum value (65599)")]
    TooBig,
}

impl TryFrom<u32> for ChunkStreamId {
    type Error = ChunkStreamIdTryFromError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 | 1 => Err(ChunkStreamIdTryFromError::Reserved),
            2..=63 => Ok(Self::OneByte(value as u8)),
            64..=319 => Ok(Self::TwoBytes((value - 64) as u8)),
            320..=65599 => Ok(Self::ThreeBytes((value - 64) as u16)),
            65600.. => Err(ChunkStreamIdTryFromError::TooBig),
        }
    }
}

impl From<ChunkStreamId> for u32 {
    fn from(value: ChunkStreamId) -> Self {
        match value {
            ChunkStreamId::OneByte(id) => id as u32,
            ChunkStreamId::TwoBytes(id) => id as u32 + 64,
            ChunkStreamId::ThreeBytes(id) => id as u32 + 64,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod chunk_stream_id {
        use super::*;

        #[test]
        fn try_from_u32() {
            // reserved values
            ChunkStreamId::try_from(0).expect_err("reserved");
            ChunkStreamId::try_from(1).expect_err("reserved");

            // too-big values
            ChunkStreamId::try_from(65600).expect_err("too big");
            ChunkStreamId::try_from(u32::MAX).expect_err("too big");
        }

        #[test]
        fn into_u32() {
            assert_eq!(u32::from(ChunkStreamId::OneByte(2)), 2u32);
            assert_eq!(u32::from(ChunkStreamId::TwoBytes(200)), 264u32);
            assert_eq!(u32::from(ChunkStreamId::ThreeBytes(10000)), 10064);
        }
    }

    #[test]
    fn write() {
        // one-byte values
        let hdr = BasicHeader {
            chunk_type: 0,
            chunk_stream_id: ChunkStreamId::try_from(2).unwrap(),
        }
        .to_bytes()
        .unwrap();
        assert_eq!(hdr.len(), 1);
        assert_eq!(hdr[0], 2);

        let hdr = BasicHeader {
            chunk_type: 3,
            chunk_stream_id: ChunkStreamId::try_from(63).unwrap(),
        }
        .to_bytes()
        .unwrap();
        assert_eq!(hdr.len(), 1);
        assert_eq!(hdr[0], u8::MAX);

        // two-byte values
        let hdr = BasicHeader {
            chunk_type: 0,
            chunk_stream_id: ChunkStreamId::try_from(64).unwrap(),
        }
        .to_bytes()
        .unwrap();
        assert_eq!(hdr.len(), 2);
        assert_eq!(hdr.as_slice(), &[0, 0]);

        let hdr = BasicHeader {
            chunk_type: 3,
            chunk_stream_id: ChunkStreamId::try_from(319).unwrap(),
        }
        .to_bytes()
        .unwrap();
        assert_eq!(hdr.len(), 2);
        assert_eq!(hdr.as_slice(), &((3 << 14) | 255u16).to_be_bytes());

        // three-byte values
        let hdr = BasicHeader {
            chunk_type: 0,
            chunk_stream_id: ChunkStreamId::try_from(320).unwrap(),
        }
        .to_bytes()
        .unwrap();
        assert_eq!(hdr.len(), 3);
        assert_eq!(hdr.as_slice(), &[1, 1, 0]);

        let hdr = BasicHeader {
            chunk_type: 0,
            chunk_stream_id: ChunkStreamId::try_from(65599).unwrap(),
        }
        .to_bytes()
        .unwrap();
        assert_eq!(hdr.len(), 3);
        assert_eq!(hdr.as_slice(), &[1, u8::MAX, u8::MAX]);
    }
}
