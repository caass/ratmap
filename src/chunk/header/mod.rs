use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::Endian,
    prelude::*,
};
use thiserror::Error;

mod basic;
mod message;

use super::ChunkStreamMap;
use basic::{BasicHeader, ChunkStreamIdTryFromError};
use message::MessageHeader;

#[derive(Debug, PartialEq)]
pub struct Header {
    basic_header: BasicHeader,
    message_header: MessageHeader,
    extended_timestamp: Option<u32>,
}

impl<'input, 'ctx> DekuRead<'input, &'ctx ChunkStreamMap> for Header {
    fn read(
        input: &'input BitSlice<u8, Msb0>,
        ctx: &'ctx ChunkStreamMap,
    ) -> Result<(&'input BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized,
    {
        let (input, basic_header) = BasicHeader::read(input, ())?;
        let (input, message_header) = MessageHeader::read(input, basic_header.chunk_type)?;

        let last_timestamp = ctx
            .get(&basic_header.chunk_stream_id())
            .map(|info| info.last_timestamp)
            .unwrap_or_default();

        let (input, extended_timestamp) = if message_header.has_extended_timestamp(last_timestamp) {
            u32::read(input, Endian::Big).map(|(input, ts)| (input, Some(ts)))?
        } else {
            (input, None)
        };

        Ok((
            input,
            Header {
                basic_header,
                message_header,
                extended_timestamp,
            },
        ))
    }
}

impl DekuWrite for Header {
    fn write(&self, output: &mut BitVec<u8, Msb0>, ctx: ()) -> Result<(), DekuError> {
        self.basic_header.write(output, ctx)?;
        self.message_header
            .write(output, self.basic_header.chunk_type)?;
        self.extended_timestamp.write(output, Endian::Big)
    }
}

impl Header {
    pub fn size(&self) -> usize {
        self.basic_header.size()
            + self.message_header.size()
            + if self.extended_timestamp.is_some() {
                4
            } else {
                0
            }
    }

    /// Get the ID of the chunk stream this chunk belongs to
    pub fn chunk_stream_id(&self) -> u32 {
        self.basic_header.chunk_stream_id()
    }

    /// Get the timestamp of this chunk, given the previous timestamp.
    /// The returned timestamp will either be an absolute value (`Timestamp::Absolute(n)`)
    /// or a delta (`Timestamp::Delta(n)`) representing an increase of `n` milliseconds.
    pub(super) fn timestamp(&self, map: &ChunkStreamMap) -> Timestamp {
        match self.message_header {
            MessageHeader::BeginOrRewindStream { timestamp, .. } => Timestamp::Absolute(timestamp),
            MessageHeader::BeginVariableLengthMessage {
                timestamp_delta, ..
            }
            | MessageHeader::BeginConstantLengthMessage { timestamp_delta } => {
                Timestamp::Delta(timestamp_delta)
            }
            MessageHeader::ContinueMessage => Timestamp::Delta(
                map.get(&self.basic_header.chunk_stream_id())
                    .expect("Received a Type 3 message with no prior message in chunk stream.")
                    .last_timestamp,
            ),
        }
    }

    /// Get the length of the message this chunk is a part of. Returns `None` if this isn't the first
    /// chunk sent for a message.
    pub fn message_length(&self) -> Option<u32> {
        match self.message_header {
            MessageHeader::BeginOrRewindStream { message_length, .. }
            | MessageHeader::BeginVariableLengthMessage { message_length, .. } => {
                Some(message_length)
            }
            _ => None,
        }
    }

    /// Get the length of the message this chunk is a part of. Returns `None` if this isn't the first
    /// chunk sent for a message.
    pub fn message_type_id(&self) -> Option<u8> {
        match self.message_header {
            MessageHeader::BeginOrRewindStream {
                message_type_id, ..
            }
            | MessageHeader::BeginVariableLengthMessage {
                message_type_id, ..
            } => Some(message_type_id),
            _ => None,
        }
    }

    pub fn message_stream_id(&self) -> Option<u32> {
        if let MessageHeader::BeginOrRewindStream {
            message_stream_id, ..
        } = self.message_header
        {
            Some(message_stream_id)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum Timestamp {
    Delta(u32),
    Absolute(u32),
}

impl Timestamp {
    pub fn into_inner(self) -> u32 {
        match self {
            Self::Delta(inner) | Self::Absolute(inner) => inner,
        }
    }
}

// constructors
impl Header {
    /// Construct a chunk header for a new chunk stream or after rewinding a stream
    pub fn begin_or_rewind_stream(
        chunk_stream_id: u32,
        timestamp: u32,
        message_length: u32,
        message_type_id: u8,
        message_stream_id: u32,
    ) -> Result<Self, ChunkHeaderError> {
        if message_length > 0xFFFFFF {
            return Err(ChunkHeaderError::MessageTooLong);
        };

        let basic_header = BasicHeader::begin_or_rewind_stream(chunk_stream_id)?;

        let (timestamp, extended_timestamp) = if timestamp >= 0xFFFFFF {
            (0xFFFFFF, Some(timestamp))
        } else {
            (timestamp, None)
        };

        let message_header = MessageHeader::BeginOrRewindStream {
            timestamp,
            message_length,
            message_type_id,
            message_stream_id,
        };

        Ok(Self {
            basic_header,
            message_header,
            extended_timestamp,
        })
    }

    /// Construct a chunk header for a new message in a stream where messages have variable length and/or types
    pub fn begin_variable_length_message(
        chunk_stream_id: u32,
        timestamp_delta: u32,
        message_length: u32,
        message_type_id: u8,
    ) -> Result<Self, ChunkHeaderError> {
        if message_length > 0xFFFFFF {
            return Err(ChunkHeaderError::MessageTooLong);
        };

        let basic_header = BasicHeader::begin_variable_length_message(chunk_stream_id)?;

        let (timestamp_delta, extended_timestamp) = if timestamp_delta >= 0xFFFFFF {
            (0xFFFFFF, Some(timestamp_delta))
        } else {
            (timestamp_delta, None)
        };

        let message_header = MessageHeader::BeginVariableLengthMessage {
            timestamp_delta,
            message_length,
            message_type_id,
        };

        Ok(Self {
            basic_header,
            message_header,
            extended_timestamp,
        })
    }

    /// Construct a chunk header for a new message in a stream where messages have constant length
    pub fn begin_constant_length_message(
        chunk_stream_id: u32,
        timestamp_delta: u32,
    ) -> Result<Self, ChunkHeaderError> {
        let basic_header = BasicHeader::begin_constant_length_message(chunk_stream_id)?;

        let (timestamp_delta, extended_timestamp) = if timestamp_delta >= 0xFFFFFF {
            (0xFFFFFF, Some(timestamp_delta))
        } else {
            (timestamp_delta, None)
        };

        let message_header = MessageHeader::BeginConstantLengthMessage { timestamp_delta };

        Ok(Self {
            basic_header,
            message_header,
            extended_timestamp,
        })
    }

    /// Construct a chunk header to continue an existing message
    pub fn continue_message(chunk_stream_id: u32) -> Result<Self, ChunkHeaderError> {
        Ok(Self {
            basic_header: BasicHeader::continue_message(chunk_stream_id)?,
            message_header: MessageHeader::ContinueMessage,
            extended_timestamp: None,
        })
    }
}

#[derive(Debug, Error)]
pub enum ChunkHeaderError {
    #[error(transparent)]
    ChunkStreamIdError(#[from] ChunkStreamIdTryFromError),
    #[error("Cannot construct a chunk header for a message longer than 0xFFFFFF bytes.")]
    MessageTooLong,
}
