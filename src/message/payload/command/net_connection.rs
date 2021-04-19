#[derive(Debug, PartialEq)]
pub enum NetConnectionCommand {
    Connect,
    Call,
    Close,
    CreateStream,
}
