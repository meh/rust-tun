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

use crate::platform::posix::Fd;
use crate::PACKET_INFORMATION_LENGTH as PIL;
use bytes::BufMut;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::sync::Arc;

/// Infer the protocol based on the first nibble in the packet buffer.
pub(crate) fn is_ipv6(buf: &[u8]) -> std::io::Result<bool> {
    use std::io::{Error, ErrorKind::InvalidData};
    if buf.is_empty() {
        return Err(Error::new(InvalidData, "Zero-length data"));
    }
    match buf[0] >> 4 {
        4 => Ok(false),
        6 => Ok(true),
        p => Err(Error::new(InvalidData, format!("IP version {}", p))),
    }
}

pub(crate) fn generate_packet_information(
    _packet_information: bool,
    _ipv6: bool,
) -> Option<[u8; PIL]> {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TUN_PROTO_IP6: [u8; PIL] = (libc::ETH_P_IPV6 as u32).to_be_bytes();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const TUN_PROTO_IP4: [u8; PIL] = (libc::ETH_P_IP as u32).to_be_bytes();

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    const TUN_PROTO_IP6: [u8; PIL] = (libc::AF_INET6 as u32).to_be_bytes();
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    const TUN_PROTO_IP4: [u8; PIL] = (libc::AF_INET as u32).to_be_bytes();

    // FIXME: Currently, the FreeBSD we test (FreeBSD-14.0-RELEASE) seems to have no PI. Here just a dummy.
    #[cfg(target_os = "freebsd")]
    const TUN_PROTO_IP6: [u8; PIL] = 0x86DD_u32.to_be_bytes();
    #[cfg(target_os = "freebsd")]
    const TUN_PROTO_IP4: [u8; PIL] = 0x0800_u32.to_be_bytes();

    #[cfg(unix)]
    if _packet_information {
        if _ipv6 {
            return Some(TUN_PROTO_IP6);
        } else {
            return Some(TUN_PROTO_IP4);
        }
    }
    None
}

/// Read-only end for a file descriptor.
pub struct Reader {
    pub(crate) fd: Arc<Fd>,
    pub(crate) offset: usize,
    pub(crate) buf: Vec<u8>,
    pub(crate) mtu: u16,
}

impl Reader {
    pub(crate) fn set_mtu(&mut self, value: u16) {
        self.mtu = value;
        self.buf.resize(value as usize + self.offset, 0);
    }

    pub(crate) fn recv(&self, mut in_buf: &mut [u8]) -> io::Result<usize> {
        const STACK_BUF_LEN: usize = crate::DEFAULT_MTU as usize + PIL;
        let in_buf_len = in_buf.len() + self.offset;

        // The following logic is to prevent dynamically allocating Vec on every recv
        // As long as the MTU is set to value lesser than 1500, this api uses `stack_buf`
        // and avoids `Vec` allocation
        let mut dynamic_buf: Vec<u8>;
        let mut fixed_buf: [u8; STACK_BUF_LEN];
        let local_buf = if in_buf_len > STACK_BUF_LEN && self.offset != 0 {
            dynamic_buf = vec![0u8; in_buf_len];
            &mut dynamic_buf[..]
        } else {
            fixed_buf = [0u8; STACK_BUF_LEN];
            &mut fixed_buf
        };

        let either_buf = if self.offset != 0 {
            &mut *local_buf
        } else {
            &mut *in_buf
        };
        let amount = self.fd.read(either_buf)?;
        if self.offset != 0 {
            in_buf.put_slice(&local_buf[self.offset..amount]);
        }
        Ok(amount - self.offset)
    }
}

impl Read for Reader {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let either_buf = if self.offset != 0 {
            let len = buf.len() + self.offset;
            if len > self.buf.len() {
                self.buf.resize(len, 0_u8);
            }
            &mut self.buf[..len]
        } else {
            &mut *buf
        };
        let amount = self.fd.read(either_buf)?;
        if self.offset != 0 {
            buf.put_slice(&self.buf[self.offset..amount]);
        }
        Ok(amount - self.offset)
    }
}

impl AsRawFd for Reader {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

/// Write-only end for a file descriptor.
pub struct Writer {
    pub(crate) fd: Arc<Fd>,
    pub(crate) offset: usize,
    pub(crate) buf: Vec<u8>,
    pub(crate) mtu: u16,
}

impl Writer {
    pub(crate) fn set_mtu(&mut self, value: u16) {
        self.mtu = value;
        self.buf.resize(value as usize + self.offset, 0);
    }

    pub(crate) fn send(&self, in_buf: &[u8]) -> io::Result<usize> {
        const STACK_BUF_LEN: usize = crate::DEFAULT_MTU as usize + PIL;
        let in_buf_len = in_buf.len() + self.offset;

        // The following logic is to prevent dynamically allocating Vec on every send
        // As long as the MTU is set to value lesser than 1500, this api uses `stack_buf`
        // and avoids `Vec` allocation
        let mut dynamic_buf: Vec<u8>;
        let mut fixed_buf: [u8; STACK_BUF_LEN];
        let local_buf = if in_buf_len > STACK_BUF_LEN && self.offset != 0 {
            dynamic_buf = vec![0_u8; in_buf_len];
            &mut dynamic_buf[..]
        } else {
            fixed_buf = [0_u8; STACK_BUF_LEN];
            &mut fixed_buf
        };

        let either_buf = if self.offset != 0 {
            let ipv6 = is_ipv6(in_buf)?;
            if let Some(header) = generate_packet_information(true, ipv6) {
                (&mut local_buf[..self.offset]).put_slice(header.as_ref());
                (&mut local_buf[self.offset..in_buf_len]).put_slice(in_buf);
                local_buf
            } else {
                in_buf
            }
        } else {
            in_buf
        };
        let amount = self.fd.write(either_buf)?;
        Ok(amount - self.offset)
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let buf = if self.offset != 0 {
            let ipv6 = is_ipv6(buf)?;
            if let Some(header) = generate_packet_information(true, ipv6) {
                let len = self.offset + buf.len();
                if len > self.buf.len() {
                    self.buf.resize(len, 0_u8);
                }
                (&mut self.buf[..self.offset]).put_slice(header.as_ref());
                (&mut self.buf[self.offset..len]).put_slice(buf);
                &self.buf[..len]
            } else {
                buf
            }
        } else {
            buf
        };
        let amount = self.fd.write(buf)?;
        Ok(amount - self.offset)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsRawFd for Writer {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

pub struct Tun {
    pub(crate) reader: Reader,
    pub(crate) writer: Writer,
    pub(crate) mtu: u16,
    pub(crate) packet_information: bool,
}

impl Tun {
    pub(crate) fn new(fd: Fd, mtu: u16, packet_information: bool) -> Self {
        let fd = Arc::new(fd);
        let offset = if packet_information { PIL } else { 0 };
        Self {
            reader: Reader {
                fd: fd.clone(),
                offset,
                buf: vec![0; mtu as usize + offset],
                mtu,
            },
            writer: Writer {
                fd,
                offset,
                buf: vec![0; mtu as usize + offset],
                mtu,
            },
            mtu,
            packet_information,
        }
    }

    pub fn set_nonblock(&self) -> io::Result<()> {
        self.reader.fd.set_nonblock()
    }

    pub fn set_mtu(&mut self, value: u16) {
        self.mtu = value;
        self.reader.set_mtu(value);
        self.writer.set_mtu(value);
    }

    pub fn mtu(&self) -> u16 {
        self.mtu
    }

    pub fn packet_information(&self) -> bool {
        self.packet_information
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.recv(buf)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.writer.send(buf)
    }
}

impl Read for Tun {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Write for Tun {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl AsRawFd for Tun {
    fn as_raw_fd(&self) -> RawFd {
        self.reader.as_raw_fd()
    }
}

impl IntoRawFd for Tun {
    fn into_raw_fd(self) -> RawFd {
        drop(self.writer);
        // guarantee fd is the unique owner such that Arc::into_inner can return some
        let fd = Arc::into_inner(self.reader.fd).unwrap(); // panic if accident
        fd.into_raw_fd()
    }
}
