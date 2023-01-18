use deku::prelude::*;

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(ctx = "message_type: u8", id = "message_type")]
pub enum ProtocolControlMessage {
    /// Protocol control message 1, Set Chunk Size, is used to notify the
    /// peer of a new maximum chunk size.
    ///
    /// The maximum chunk size defaults to 128 bytes, but the client or the
    /// server can change this value, and updates its peer using this
    /// message. For example, suppose a client wants to send 131 bytes of
    /// audio data and the chunk size is 128. In this case, the client can
    /// send this message to the server to notify it that the chunk size is
    /// now 131 bytes. The client can then send the audio data in a single
    /// chunk.
    ///
    /// The maximum chunk size SHOULD be at least 128 bytes, and MUST be at
    /// least 1 byte. The maximum chunk size is maintained independently for
    /// each direction.
    #[deku(id = "1")]
    SetChunkSize(#[deku(bits = 31, pad_bits_before = "1")] u32),

    /// Protocol control message 2, Abort Message, is used to notify the peer
    /// if it is waiting for chunks to complete a message, then to discard
    /// the partially received message over a chunk stream. The peer
    /// receives the chunk stream ID as this protocol message’s payload. An
    /// application may send this message when closing in order to indicate
    /// that further processing of the messages is not required.
    #[deku(id = "2")]
    AbortMessage { message_stream_id: u32 },

    /// The client or the server MUST send an acknowledgment to the peer
    /// after receiving bytes equal to the window size. The window size is
    /// the maximum number of bytes that the sender sends without receiving
    /// acknowledgment from the receiver. This message specifies the
    /// sequence number, which is the number of the bytes received so far.
    #[deku(id = "3")]
    Acknowledgement { sequence_number: u32 },

    /// The client or the server sends this message to inform the peer of the
    /// window size to use between sending acknowledgments. The sender
    /// expects acknowledgment from its peer after the sender sends window
    /// size bytes. The receiving peer MUST send an Acknowledgement
    /// (Section 5.4.3) after receiving the indicated number of bytes since
    /// the last Acknowledgement was sent, or from the beginning of the
    /// session if no Acknowledgement has yet been sent.
    #[deku(id = "5")]
    WindowAcknowledgementSize(u32),

    /// The client or the server sends this message to limit the output
    /// bandwidth of its peer. The peer receiving this message limits its
    /// output bandwidth by limiting the amount of sent but unacknowledged
    /// data to the window size indicated in this message. The peer
    /// receiving this message SHOULD respond with a Window Acknowledgement
    /// Size message if the window size is different from the last one sent
    /// to the sender of this message.
    #[deku(id = "6")]
    SetPeerBandwidth {
        acknowledgement_window_size: u32,
        limit_type: LimitType,
    },
}

#[derive(Debug, PartialEq, Eq, DekuRead, DekuWrite)]
#[deku(type = "u8")]
pub enum LimitType {
    /// The peer SHOULD limit its output bandwidth to the
    /// indicated window size.
    #[deku(id = "0")]
    Hard,

    /// The peer SHOULD limit its output bandwidth to the the
    /// window indicated in this message or the limit already in effect,
    /// whichever is smaller.
    #[deku(id = "1")]
    Soft,

    /// If the previous Limit Type was Hard, treat this message
    /// as though it was marked Hard, otherwise ignore this message.
    #[deku(id = "2")]
    Dynamic,
}
