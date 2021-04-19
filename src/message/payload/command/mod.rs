use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    prelude::*,
};

#[derive(Debug, PartialEq)]
pub struct CommandMessage {
    transaction_id: u32,
    command: Command,
}

#[derive(Debug, PartialEq)]
pub enum Command {
    NetConnection(NetConnectionCommand),
    NetStream(NetStreamCommand),
    Response(CommandResponse),
}

#[derive(Debug, PartialEq)]
pub enum NetConnectionCommand {
    Connect,
    Call,
    Close,
    CreateStream,
}
#[derive(Debug, PartialEq)]
pub enum NetStreamCommand {
    Play,
    Play2,
    DeleteStream,
    CloseStream,
    ReceiveAudio,
    ReceiveVideo,
    Publish,
    Seek,
    Pause,
    OnStatus,
}

#[derive(Debug, PartialEq)]
pub enum CommandResponse {
    Result,
    Error,
    NetConnectionCommand(NetConnectionCommand),
    NetStreamCommand(NetStreamCommand),
}

impl DekuRead<'_, (u8, u32)> for CommandMessage {
    fn read(
        input: &BitSlice<Msb0, u8>,
        (message_type, payload_length): (u8, u32),
    ) -> Result<(&BitSlice<Msb0, u8>, Self), DekuError>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl DekuWrite<(u8, u32)> for CommandMessage {
    fn write(
        &self,
        output: &mut BitVec<Msb0, u8>,
        (message_type, payload_length): (u8, u32),
    ) -> Result<(), DekuError> {
        todo!()
    }
}
