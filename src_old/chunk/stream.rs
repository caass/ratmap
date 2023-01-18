use super::Chunk;
use crate::{error::RatmapResult, StreamSetting};
use async_trait::async_trait;
use deku::bitvec::{BitSlice, BitVec, Msb0};

#[async_trait]
pub trait TcpStream {
    async fn read_chunk(
        &self,
        chunk_size: u32,
        leftover_bits: &mut BitSlice<Msb0, u8>,
    ) -> RatmapResult<(BitVec<Msb0, u8>, Chunk)>;
    async fn write_chunk(&self, chunk: Chunk, chunk_size: u32) -> RatmapResult<()>;
    async fn close(self) -> RatmapResult<()>;
}

pub struct ChunkStream<T: TcpStream> {
    /// Underlying TCP stream
    inner: T,

    // TODO: this should probably be a Arc<RwLock<BitVec<Msb0, u8>>> or Arc<Mutex<BitVec<Msb0, u8>>> to avoid copies
    /// Any bits that were read but not used in parsing a chunk
    leftover_bits: BitVec<Msb0, u8>,
}

impl<T: TcpStream> ChunkStream<T> {
    /// Construct a new ChunkStream from an underlying TCP Stream.
    /// Implementations are provided for various crates, which can be
    /// opted-into by activating a feature of the same name. e.g.:
    ///
    /// ```no_rust
    /// # Cargo.toml
    /// ratmap = { version = "*", features = ["tokio"] }
    /// ```
    ///
    /// allows the use of `tokio::io::TcpStream`.
    pub fn new(tcp_stream: T) -> Self {
        Self {
            inner: tcp_stream,
            leftover_bits: BitVec::new(),
        }
    }

    pub async fn read(&mut self, chunk_size: u32) -> RatmapResult<Chunk> {
        let (leftover, chunk) = self
            .inner
            .read_chunk(chunk_size, &mut self.leftover_bits)
            .await?;

        self.leftover_bits = leftover;
        Ok(chunk)
    }

    pub async fn write(&self, chunk: Chunk, chunk_size: u32) -> RatmapResult<()> {
        self.inner.write_chunk(chunk, chunk_size).await
    }

    /// Close the ChunkStream so it can be dropped cleanly
    pub async fn close(self) -> RatmapResult<()> {
        self.inner.close().await
    }

    /// Yields the underlying TCP stream, consuming the ChunkStream
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Yields a reference to the underlying TCP stream
    pub fn inner(&self) -> &T {
        &self.inner
    }
}
