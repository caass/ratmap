use std::hash::BuildHasherDefault;
use std::io;

use dashmap::DashMap;
use rustc_hash::FxHasher;
use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};

use crate::clock::{Clock, SystemClock};

mod handshake;
mod header;

use self::handshake::handshake;
use self::header::Header;

#[derive(Debug)]
struct Chunk {
    header: Header,
    data: Vec<u8>,
}

#[derive(Debug)]
struct ChunkStreamInformation {
    last_timestamp: u32,
    maximum_chunk_size: u32,
    message_stream_id: u32,
}

type ChunkStreamMap = DashMap<u32, ChunkStreamInformation, BuildHasherDefault<FxHasher>>;

pub struct ChunkStream<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send,
    C: Clock,
{
    reader: ReadHalf<RW>,
    writer: WriteHalf<RW>,
    clock: C,
    incoming: ChunkStreamMap,
    outgoing: ChunkStreamMap,
}

impl<RW> ChunkStream<RW, SystemClock>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub async fn new(stream: RW) -> io::Result<Self> {
        Self::new_with_clock(stream, SystemClock::default()).await
    }

    async fn read_chunk(&mut self) -> io::Result<Chunk> {
        todo!()
    }
}

impl<RW, C> ChunkStream<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send,
    C: Clock,
{
    pub async fn new_with_clock(mut stream: RW, clock: C) -> io::Result<Self> {
        let _approximate_latency = handshake(&mut stream, &clock).await?;
        let (reader, writer) = split(stream);

        Ok(Self {
            reader,
            writer,
            clock,
            incoming: ChunkStreamMap::default(),
            outgoing: ChunkStreamMap::default(),
        })
    }

    pub fn into_inner(self) -> RW {
        self.reader.unsplit(self.writer)
    }
}
