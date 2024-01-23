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

use crate::PACKET_INFORMATION_LENGTH;
use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Infer the protocol based on the first nibble in the packet buffer.
fn is_ipv6(buf: &[u8]) -> std::io::Result<bool> {
    use std::io::{Error, ErrorKind::InvalidData};
    match buf[0] >> 4 {
        4 => Ok(false),
        6 => Ok(true),
        p => Err(Error::new(InvalidData, format!("IP version {}", p))),
    }
}

fn generate_packet_information(_packet_information: bool, _ipv6: bool) -> Option<Bytes> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TUN_PROTO_IP6: [u8; PACKET_INFORMATION_LENGTH] = (libc::ETH_P_IPV6 as u32).to_be_bytes();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TUN_PROTO_IP4: [u8; PACKET_INFORMATION_LENGTH] = (libc::ETH_P_IP as u32).to_be_bytes();

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    const TUN_PROTO_IP6: [u8; PACKET_INFORMATION_LENGTH] = (libc::AF_INET6 as u32).to_be_bytes();
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    const TUN_PROTO_IP4: [u8; PACKET_INFORMATION_LENGTH] = (libc::AF_INET as u32).to_be_bytes();

    #[cfg(unix)]
    if _packet_information {
        let mut buf = BytesMut::with_capacity(PACKET_INFORMATION_LENGTH);
        if _ipv6 {
            buf.put_slice(&TUN_PROTO_IP6);
        } else {
            buf.put_slice(&TUN_PROTO_IP4);
        }
        return Some(buf.freeze());
    }
    None
}

/// A Tun Packet to be sent or received on the TUN interface.
#[derive(Debug)]
pub struct TunPacket {
    /// The packet information header.
    pub(crate) header: Option<Bytes>,
    /// The packet bytes.
    pub(crate) bytes: Bytes,
}

impl TunPacket {
    /// Create a new `TunPacket` based on a byte slice.
    pub fn new<S: AsRef<[u8]> + Into<Bytes>>(packet_information: bool, bytes: S) -> TunPacket {
        let bytes = bytes.into();
        let ipv6 = is_ipv6(bytes.as_ref()).unwrap();
        let header = generate_packet_information(packet_information, ipv6);
        TunPacket { header, bytes }
    }

    /// Return this packet's bytes.
    pub fn get_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }
}

/// A TunPacket Encoder/Decoder.
#[derive(Debug, Default)]
pub struct TunPacketCodec {
    /// Whether the underlying tunnel Device has enabled the packet information header.
    pub(crate) packet_information: bool,

    /// The MTU of the underlying tunnel Device.
    pub(crate) mtu: usize,
}

impl TunPacketCodec {
    /// Create a new `TunPacketCodec` specifying whether the underlying
    ///  tunnel Device has enabled the packet information header.
    pub fn new(packet_information: bool, mtu: usize) -> TunPacketCodec {
        let mtu = u16::try_from(mtu).unwrap_or(crate::DEFAULT_MTU as u16) as usize;
        TunPacketCodec {
            packet_information,
            mtu,
        }
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
        if self.packet_information {
            buf.reserve(self.mtu + PACKET_INFORMATION_LENGTH);
        } else {
            buf.reserve(self.mtu);
        }

        let mut header = None;
        // if the packet information is enabled we have to ignore the first 4 bytes
        if self.packet_information {
            header = Some(pkt.split_to(PACKET_INFORMATION_LENGTH).into());
        }

        let bytes = pkt.freeze();
        Ok(Some(TunPacket { header, bytes }))
    }
}

impl Encoder<TunPacket> for TunPacketCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: TunPacket, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let extra = PACKET_INFORMATION_LENGTH;
        dst.reserve(item.get_bytes().len() + if self.packet_information { extra } else { 0 });
        if self.packet_information {
            if let Some(header) = &item.header {
                dst.put_slice(header.as_ref());
            }
        }
        dst.put(item.get_bytes());
        Ok(())
    }
}
