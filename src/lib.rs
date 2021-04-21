#![allow(clippy::clippy::module_inception)] // TODO: think of a better name than chunk/chunk and message/message

mod chunk;
mod error;
mod handshake;
mod message;

struct StreamSetting<T> {
    incoming: T,
    outgoing: T,
}
