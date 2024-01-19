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

use core::pin::Pin;
use core::task::{Context, Poll};
use futures_core::ready;
use std::io::{IoSlice, Read, Write};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::codec::Framed;

use crate::codec::TunPacketCodec;
use crate::device::AbstractDevice;
use crate::platform::Device;

/// An async TUN device wrapper around a TUN device.
pub struct AsyncDevice {
    inner: AsyncFd<Device>,
}

impl AsyncDevice {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> std::io::Result<AsyncDevice> {
        device.set_nonblock()?;
        Ok(AsyncDevice {
            inner: AsyncFd::new(device)?,
        })
    }
    /// Returns a shared reference to the underlying Device object
    pub fn get_ref(&self) -> &Device {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying Device object
    pub fn get_mut(&mut self) -> &mut Device {
        self.inner.get_mut()
    }

    /// Consumes this AsyncDevice and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let packet_information = self.get_ref().has_packet_information();
        let mtu = self.inner.get_ref().mtu().unwrap_or(1504);
        let codec = TunPacketCodec::new(packet_information, mtu);
        // associate mtu with the capacity of ReadBuf
        Framed::with_capacity(self, codec, mtu as usize)
    }
}

impl AsyncRead for AsyncDevice {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf,
    ) -> Poll<std::io::Result<()>> {
        loop {
            let mut guard = ready!(self.inner.poll_read_ready_mut(cx))?;
            let rbuf = buf.initialize_unfilled();
            match guard.try_io(|inner| inner.get_mut().read(rbuf)) {
                Ok(res) => return Poll::Ready(res.map(|n| buf.advance(n))),
                Err(_wb) => continue,
            }
        }
    }
}

impl AsyncWrite for AsyncDevice {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;
            match guard.try_io(|inner| inner.get_mut().write(buf)) {
                Ok(res) => return Poll::Ready(res),
                Err(_wb) => continue,
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;
            match guard.try_io(|inner| inner.get_mut().flush()) {
                Ok(res) => return Poll::Ready(res),
                Err(_wb) => continue,
            }
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        loop {
            let mut guard = ready!(self.inner.poll_write_ready_mut(cx))?;
            match guard.try_io(|inner| inner.get_mut().write_vectored(bufs)) {
                Ok(res) => return Poll::Ready(res),
                Err(_wb) => continue,
            }
        }
    }

    fn is_write_vectored(&self) -> bool {
        true
    }
}
