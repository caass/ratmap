use deku::{
    bitvec::{BitSlice, BitVec, Msb0},
    prelude::*,
};
use std::{cmp::Ordering, io};

use async_trait::async_trait;

use tokio::io::AsyncWriteExt;

use crate::chunk::{Chunk, TcpStream};
use crate::error::{RatmapError, RatmapResult};

#[async_trait]
impl TcpStream for tokio::net::TcpStream {
    async fn read_chunk(
        &self,
        chunk_size: u32,
        leftover_bits: &mut BitSlice<Msb0, u8>,
    ) -> RatmapResult<(BitVec<Msb0, u8>, Chunk)> {
        // if we have enough bits, try just reading them
        if leftover_bits.len() / 8 >= chunk_size as usize {
            if let Ok((rest, chunk)) = Chunk::read(&leftover_bits, chunk_size) {
                return Ok((rest.to_bitvec(), chunk));
            }
        }

        // 1-copy, is it possible to get this to zero-copy?
        let mut bytes = leftover_bits.to_bitvec().into_vec();
        let num_existing_bytes = bytes.len();
        let num_bytes_expected = chunk_size as usize + 18 - num_existing_bytes;

        bytes.reserve_exact(num_bytes_expected);

        let buf = &mut bytes[num_existing_bytes..];

        loop {
            // Wait for the socket to be readable
            self.readable().await.map_err(RatmapError::Io)?;

            // Try to read data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match self.try_read(buf) {
                Ok(num_bytes_read) => {
                    if num_bytes_read == 0 {
                        return Err(RatmapError::Io(io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            "Connection is closed, no more chunks can be read!",
                        )));
                    }

                    let mut bits = BitVec::<Msb0, _>::from_vec(bytes);
                    return match Chunk::read(&bits, chunk_size) {
                        Ok((rest, chunk)) => Ok((rest.to_bitvec(), chunk)),
                        Err(e) => {
                            match e {
                                // we didn't have enough bits
                                DekuError::Incomplete(_) => {
                                    return self.read_chunk(chunk_size, &mut bits).await
                                }
                                other_error => return Err(RatmapError::Deku(other_error)),
                            }
                        }
                    };
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    async fn write_chunk(&self, chunk: Chunk, chunk_size: u32) -> RatmapResult<()> {
        let mut bits = BitVec::<Msb0, u8>::with_capacity((chunk_size as usize + 14) * 8);
        chunk
            .write(&mut bits, chunk_size)
            .map_err(RatmapError::Deku)?;
        let buf = bits.into_vec();

        let num_bytes = buf.len();

        loop {
            // Wait for the socket to be writable
            self.writable().await.map_err(RatmapError::Io)?;

            // Try to write data, this may still fail with `WouldBlock`
            // if the readiness event is a false positive.
            match self.try_write(&buf) {
                Ok(num_bytes_written) => {
                    return match num_bytes_written.cmp(&num_bytes) {
                        Ordering::Less => {
                            // we didn't write enough bytes
                            return Err(RatmapError::Io(io::Error::new(
                                io::ErrorKind::UnexpectedEof,
                                format!(
                                    "Expected to write {} bytes, but only {} bytes were written!",
                                    num_bytes, num_bytes_written
                                ),
                            )));
                        }
                        Ordering::Equal => Ok(()),
                        Ordering::Greater => unreachable!(),
                    };
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
    }

    async fn close(mut self) -> RatmapResult<()> {
        // TODO: do we need to do anything to stop a chunk stream?
        self.shutdown().await.map_err(RatmapError::Io)
    }
}
