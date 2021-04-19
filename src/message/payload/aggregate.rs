///! First of all let me just say:
///! Stupid that the RTMP spec doesn't
///! actually specify the size of the back pointer.
///! I had to look [here](http://kundansingh.com/p/rtclite/vnd/adobe/rtmp.py).
///! Anyway, apparently it's u32.
use super::super::Message;
use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    ctx::Endian,
    prelude::*,
};

use std::{cmp::Ordering, convert::TryInto};

#[derive(Debug, PartialEq)]
pub struct Aggregate(Vec<Message>);

impl<'a> DekuRead<'a, (u32, u32, u32)> for Aggregate {
    fn read(
        input: &'a BitSlice<Msb0, u8>,
        (payload_length, message_stream_id, aggregate_timestamp): (u32, u32, u32),
    ) -> Result<(&'a BitSlice<Msb0, u8>, Self), DekuError>
    where
        Self: Sized,
    {
        let mut output = Vec::<Message>::new();

        let mut bytes_read = 0usize;
        let mut remaining_bits = input;

        let mut prev_length = remaining_bits.len() / 8;

        let payload_length_usize = (payload_length).try_into().unwrap();
        let mut timestamp_offset = 0u32;

        loop {
            match bytes_read.cmp(&payload_length_usize) {
                // we've read all the bytes we expected to, let's bounce
                Ordering::Equal => break Ok((remaining_bits, Aggregate(output))),

                // we read too many bytes
                Ordering::Greater => {
                    break Err(DekuError::Assertion(format!(
                        "Expected to read {} bytes, but {} bytes were read!",
                        payload_length, bytes_read
                    )))
                }

                // we haven't read enough bytes yet
                Ordering::Less => {
                    let (after_message, mut message) = Message::read(remaining_bits, ())?;
                    let message_length: u32 = (after_message.len() / 8).try_into().unwrap();

                    // The message stream ID of the aggregate message overrides the message
                    // stream IDs of the sub-messages inside the aggregate.
                    message.header.stream_id = message_stream_id;

                    // The difference between the timestamps of the aggregate message and
                    // the first sub-message is the offset used to renormalize the
                    // timestamps of the sub-messages to the stream timescale. The offset
                    // is added to each sub-message’s timestamp to arrive at the normalized
                    // stream time. The timestamp of the first sub-message SHOULD be the
                    // same as the timestamp of the aggregate message, so the offset SHOULD
                    // be zero.
                    if output.is_empty() {
                        timestamp_offset = aggregate_timestamp
                            .checked_sub(message.header.timestamp)
                            .unwrap_or(message.header.timestamp - aggregate_timestamp);
                    }

                    message.header.timestamp -= timestamp_offset;

                    // the backpointer is a u32 after the message that says how long that message was for some reason
                    // even though the message header...also says that...who designed this protocol? wtf
                    let (rest, backpointer) = u32::read(after_message, Endian::Big)?;

                    if backpointer != message_length {
                        let err = format!(
                        "Expected aggregate's submessage to be ${} bytes, but it was ${} bytes!",
                        backpointer, message_length
                    );
                        break Err(DekuError::Assertion(err));
                    } else {
                        output.push(message);

                        remaining_bits = rest;

                        let new_length = remaining_bits.len() / 8;

                        bytes_read += prev_length - new_length;

                        prev_length = new_length;
                    }
                }
            }
        }
    }
}

impl DekuWrite<(u32, u32, u32)> for Aggregate {
    fn write(&self, output: &mut BitVec<Msb0, u8>, _: (u32, u32, u32)) -> Result<(), DekuError> {
        self.0
            .iter()
            .try_for_each(|message| -> Result<(), DekuError> {
                let before = output.len() / 8;
                message.write(output, ())?;
                let after = output.len() / 8;
                let message_size: u32 = (after - before).try_into().unwrap();
                message_size.write(output, Endian::Big)
            })
    }
}
