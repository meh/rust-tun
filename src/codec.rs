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

use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// A packet protocol IP version
#[derive(Debug, Clone, Copy, Default)]
enum PacketProtocol {
    #[default]
    IPv4,
    IPv6,
    Other(u8),
}

// Note: the protocol in the packet information header is platform dependent.
impl PacketProtocol {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn into_pi_field(self) -> std::io::Result<u16> {
        match self {
            PacketProtocol::IPv4 => Ok(libc::ETH_P_IP as u16),
            PacketProtocol::IPv6 => Ok(libc::ETH_P_IPV6 as u16),
            PacketProtocol::Other(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "neither an IPv4 nor IPv6 packet",
            )),
        }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn into_pi_field(self) -> std::io::Result<u16> {
        match self {
            PacketProtocol::IPv4 => Ok(libc::PF_INET as u16),
            PacketProtocol::IPv6 => Ok(libc::PF_INET6 as u16),
            PacketProtocol::Other(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "neither an IPv4 nor IPv6 packet",
            )),
        }
    }

    #[cfg(target_os = "windows")]
    #[allow(dead_code)]
    fn into_pi_field(self) -> std::io::Result<u16> {
        unimplemented!()
    }
}

/// A Tun Packet to be sent or received on the TUN interface.
#[derive(Debug)]
pub struct TunPacket(PacketProtocol, Bytes);

/// Infer the protocol based on the first nibble in the packet buffer.
fn infer_proto(buf: &[u8]) -> PacketProtocol {
    match buf[0] >> 4 {
        4 => PacketProtocol::IPv4,
        6 => PacketProtocol::IPv6,
        p => PacketProtocol::Other(p),
    }
}

impl TunPacket {
    /// Create a new `TunPacket` based on a byte slice.
    pub fn new<S: AsRef<[u8]> + Into<Bytes>>(bytes: S) -> TunPacket {
        let proto = infer_proto(bytes.as_ref());
        TunPacket(proto, bytes.into())
    }

    /// Return this packet's bytes.
    pub fn get_bytes(&self) -> &[u8] {
        &self.1
    }

    pub fn into_bytes(self) -> Bytes {
        self.1
    }
}

/// A TunPacket Encoder/Decoder.
pub struct TunPacketCodec(bool, i32);

impl TunPacketCodec {
    /// Create a new `TunPacketCodec` specifying whether the underlying
    ///  tunnel Device has enabled the packet information header.
    pub fn new(packet_information: bool, mtu: i32) -> TunPacketCodec {
        TunPacketCodec(packet_information, mtu)
    }
}

impl Decoder for TunPacketCodec {
    type Item = TunPacket;
    type Error = std::io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        let mut pkt = buf.split_to(buf.len());

        // reserve enough space for the next packet
        if self.0 {
            buf.reserve(self.1 as usize + 4);
        } else {
            buf.reserve(self.1 as usize);
        }

        // if the packet information is enabled we have to ignore the first 4 bytes
        if self.0 {
            let _ = pkt.split_to(4);
        }

        let proto = infer_proto(pkt.as_ref());
        Ok(Some(TunPacket(proto, pkt.freeze())))
    }
}

impl Encoder<TunPacket> for TunPacketCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: TunPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(item.get_bytes().len() + if self.0 { 4 } else { 0 });
        match item {
            TunPacket(_proto, bytes) if self.0 => {
                #[cfg(unix)]
                {
                    use byteorder::{NativeEndian, NetworkEndian, WriteBytesExt};

                    // build the packet information header comprising of 2 u16
                    // fields: flags and protocol.
                    let mut buf = Vec::<u8>::with_capacity(4);

                    // flags is always 0
                    buf.write_u16::<NativeEndian>(0)?;
                    // write the protocol as network byte order
                    buf.write_u16::<NetworkEndian>(_proto.into_pi_field()?)?;

                    dst.put_slice(&buf);
                }
                dst.put(bytes);
            }
            TunPacket(_, bytes) => dst.put(bytes),
        }
        Ok(())
    }
}
