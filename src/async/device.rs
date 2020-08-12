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

use std::io;

use core::pin::Pin;
use core::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, PollEvented};
use tokio_util::codec::Framed;

use crate::platform::{Device, Queue};
use crate::r#async::codec::*;

/// An async TUN device wrapper around a TUN device.
pub struct AsyncDevice {
    inner: PollEvented<Device>,
}

impl AsyncDevice {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> io::Result<AsyncDevice> {
        device.set_nonblock()?;
        Ok(AsyncDevice {
            inner: PollEvented::new(device)?,
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
    pub fn into_framed(mut self) -> Framed<Self, TunPacketCodec> {
        let pi = self.get_mut().has_packet_information();
        let codec = TunPacketCodec::new(pi);
        Framed::new(self, codec)
    }
}

impl AsyncRead for AsyncDevice {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for AsyncDevice {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// An async TUN device queue wrapper around a TUN device queue.
pub struct AsyncQueue {
    inner: PollEvented<Queue>,
}

impl AsyncQueue {
    /// Create a new `AsyncQueue` wrapping around a `Queue`.
    pub fn new(queue: Queue) -> io::Result<AsyncQueue> {
        queue.set_nonblock()?;
        Ok(AsyncQueue {
            inner: PollEvented::new(queue)?,
        })
    }
    /// Returns a shared reference to the underlying Queue object
    pub fn get_ref(&self) -> &Queue {
        self.inner.get_ref()
    }

    /// Returns a mutable reference to the underlying Queue object
    pub fn get_mut(&mut self) -> &mut Queue {
        self.inner.get_mut()
    }

    /// Consumes this AsyncQueue and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(mut self) -> Framed<Self, TunPacketCodec> {
        let pi = self.get_mut().has_packet_information();
        let codec = TunPacketCodec::new(pi);
        Framed::new(self, codec)
    }
}

impl AsyncRead for AsyncQueue {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for AsyncQueue {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}
