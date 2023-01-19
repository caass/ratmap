use std::hash::BuildHasherDefault;
use std::io;

use dashmap::DashMap;
use deku::bitvec::{BitSlice, Msb0};
use rustc_hash::FxHasher;
use tokio::io::{
    split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, BufWriter, ReadHalf,
    WriteHalf,
};
use deku::prelude::*;

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

impl <'input, 'ctx> DekuRead<'input, &'ctx ChunkStreamMap> for Chunk {
    fn read(
        input: &'input BitSlice<u8, Msb0>,
        ctx: &'ctx ChunkStreamMap,
    ) -> Result<(&'input BitSlice<u8, Msb0>, Self), DekuError>
    where
        Self: Sized {
        let (input, header) = Header::read(input, ctx)?;
        let bytes_to_read = ctx.get(&header.chunk_stream_id()).map(|info| info.maximum_chunk_size.min(info.message_bytes_left_to_read))
    }
}

#[derive(Debug)]
struct ChunkStreamInformation {
    last_timestamp: u32,
    maximum_chunk_size: u32,
    message_stream_id: u32,
    current_message: Vec<u8>,
    message_bytes_left_to_read: u32,
}

type ChunkStreamMap = DashMap<u32, ChunkStreamInformation, BuildHasherDefault<FxHasher>>;

#[derive(Debug)]
pub struct ChunkStream<RW, C>
where
    RW: AsyncRead + AsyncWrite + Unpin + Send,
    C: Clock,
{
    reader: BufReader<ReadHalf<RW>>,
    writer: BufWriter<WriteHalf<RW>>,
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
        let (reader_inner, writer_inner) = split(stream);
        let reader = BufReader::new(reader_inner);
        let writer = BufWriter::new(writer_inner);

        Ok(Self {
            reader,
            writer,
            clock,
            incoming: ChunkStreamMap::default(),
            outgoing: ChunkStreamMap::default(),
        })
    }

    pub fn into_inner(self) -> RW {
        self.reader.into_inner().unsplit(self.writer.into_inner())
    }
}
