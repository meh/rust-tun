//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.
use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// A TUN packet Encoder/Decoder.
#[derive(Debug, Default)]
pub struct TunPacketCodec(usize);

impl TunPacketCodec {
    /// Create a new `TunPacketCodec` specifying whether the underlying
    ///  tunnel Device has enabled the packet information header.
    pub fn new(mtu: usize) -> TunPacketCodec {
        TunPacketCodec(mtu)
    }
}

impl Decoder for TunPacketCodec {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None);
        }
        let pkt = buf.split_to(buf.len());
        //reserve enough space for the next packet
        buf.reserve(self.0);
        Ok(Some(pkt.freeze().to_vec()))
    }
}

impl Encoder<Vec<u8>> for TunPacketCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = item.as_slice();
        dst.reserve(bytes.len());
        dst.put(bytes);
        Ok(())
    }
}
