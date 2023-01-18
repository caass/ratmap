//! https://rtmp.veriskope.com/docs/spec/#52handshake
//!
//! An RTMP connection begins with a handshake. The handshake is unlike the rest of the protocol; it consists of three static-sized chunks rather than consisting of variable-sized chunks with headers.

use tokio::io::{split, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::clock::Clock;
use std::io;

const RANDOM_BYTES: &[u8; 1528] = b"After adding Rust support to Linux kernel in 2021 Linux repo has been flooded with patches and pull requests from brave Rustaceans rewriting critical components in Rust to ensure their stability and memory safety that C could never guarantee. After a few painful years of code reviews and salt coming from C programmers losing their jobs left and right we have finally achieved a 100% Rust Linux kernel. Not a single kernel panic or crash has been reported ever since. In fact, the kernel was so stable that Microsoft gave up all their efforts in Windows as we know it, rewrote it in Rust, and Windows became just another distro in the Linux ecosystem. Other projects and companies soon followed the trend - if you install any Linux distro nowadays it won't come with grep, du or cat - there is only ripgrep, dust and bat. Do you use a graphical interface? Good luck using deprecated projects such as Wayland, Gnome or KDE - wayland-rs, Rsome and RDE is where it's all at. The only serious browser available is Servo and it holds 98% of the market share. Every new game released to the market, including those made by AAA developers, is using the most stable, fast and user-friendly game engine - Bevy v4.20. People love their system and how stable, safe and incredibly fast it is. Proprietary software is basically non-existent at this point. By the year 2035 every single printer, laptop, industrial robot, rocket, autonomous car, submarine, sex toy is powered by software written in Rust. And they never crash or fail. The w-";

pub async fn handshake<RW: AsyncRead + AsyncWrite + Unpin + Send>(
    stream: &mut RW,
    clock: &impl Clock,
) -> io::Result<u32> {
    let (mut r, mut w) = split(stream);

    let send_0_1 = async {
        // Stage 0: send the RTMP version we support
        w.write_u8(3).await?;

        // Stage 1: send a timestamp and some random bytes
        let timestamp = clock.now();
        w.write_u32(timestamp).await?;
        w.write_all(RANDOM_BYTES).await?;

        w.flush().await?;

        Ok(timestamp)
    };

    let recv_0_1 = async {
        let requested_version = r.read_u8().await?;
        if requested_version != 3 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Ratmap only supports RTMP version 3, but peer wants version {}",
                    requested_version
                ),
            ));
        };

        let zero = r.read_u32().await?;
        if zero != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected 0u32 from peer, received {zero} instead"),
            ));
        }

        let peer_timestamp = r.read_u32().await?;
        let mut random_bytes = [0; 1528];
        r.read_exact(&mut random_bytes).await?;
        let received_at = clock.now();

        Ok((peer_timestamp, random_bytes, received_at))
    };

    let (our_timestamp, (their_timestamp, their_random_bytes, received_at)) =
        tokio::try_join!(send_0_1, recv_0_1)?;

    // stage 2: peer timestamp, recieved_at, and random echo
    let send_2 = async {
        w.write_u32(their_timestamp).await?;
        w.write_u32(received_at).await?;
        w.write_all(&their_random_bytes).await?;
        w.flush().await
    };

    let recv_2 = async {
        let our_timestamp_echo = r.read_u32().await?;
        if our_timestamp_echo != our_timestamp {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Peer failed to echo our timestamp",
            ));
        };

        let they_received_at = r.read_u32().await?;

        let mut random_echo = [0; 1528];
        r.read_exact(&mut random_echo).await?;

        if &random_echo != RANDOM_BYTES {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Peer failed to echo random bytes",
            ))
        } else {
            Ok(they_received_at)
        }
    };

    let (_, they_received_at) = tokio::try_join!(send_2, recv_2)?;

    Ok(they_received_at - their_timestamp)
}
