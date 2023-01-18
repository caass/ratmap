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
