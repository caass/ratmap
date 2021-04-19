mod header;

use deku::prelude::*;
pub use header::ChunkHeader;

/// A Chunk, as described in [section 5.3.1] of the spec.
///
/// Messages in RTMP are typically communicated along the RTMP Chunk Stream,
/// which sits at the same layer as HTTP -- that is to say, it operates on TCP
/// and defines its own format optimized for communicating messages. Each chunk
/// may contain part of a message, a full message, or multiple messages (in the
/// case of an Aggregate Message).
///
/// [section 5.3.1]: https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf#page=11
#[derive(Debug, PartialEq, DekuRead, DekuWrite)]
#[deku(ctx = "chunk_size: u32")]
pub struct Chunk {
    pub header: ChunkHeader,

    /// The data transported in the chunk.
    #[deku(count = "chunk_size")]
    pub data: Vec<u8>,
}
