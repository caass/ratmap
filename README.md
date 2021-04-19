![Banner](banner.png)

Ratmap is a WIP pure Rust implementation of [Adobe's RTMP streaming protocol](https://wwwimages2.adobe.com/content/dam/acom/en/devnet/rtmp/pdf/rtmp_specification_1.0.pdf).

## Roadmap

- [ ] Provide basic RTMP primitives (`ChunkStream`, `MessageStream`, etc.)
- [ ] Provide TCP-provider-agnostic `RTMPClient<T>` / `RTMPServer<T>` structs where `T: TcpProvider`
- [ ] Implement `TcpProvider` for common libs
  - [ ] [`std::net::TcpStream`](https://doc.rust-lang.org/nightly/std/net/struct.TcpStream.html)
  - [ ] [`tokio::net::TcpStream`](https://docs.rs/tokio/1.5.0/tokio/net/struct.TcpStream.html)
  - [ ] [`async_std::net::TcpStream`](https://docs.rs/async-std/1.9.0/async_std/net/struct.TcpStream.html)
  - [ ] [`message_io::node`](https://docs.rs/message-io/0.13.0/message_io/node/index.html)
