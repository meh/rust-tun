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

use super::TunPacketCodec;
use crate::device::AbstractDevice;
use crate::platform::Device;
use core::pin::Pin;
use core::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::codec::Framed;
use wintun_bindings::AsyncSession;

/// An async TUN device wrapper around a TUN device.
pub struct AsyncDevice {
    inner: Device,
    session_reader: DeviceReader,
    session_writer: DeviceWriter,
}

/// Returns a shared reference to the underlying Device object.
impl core::ops::Deref for AsyncDevice {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Returns a mutable reference to the underlying Device object.
impl core::ops::DerefMut for AsyncDevice {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl AsyncDevice {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> std::io::Result<AsyncDevice> {
        let session_reader = DeviceReader::new(device.tun.get_session().into())?;
        let session_writer = DeviceWriter::new(device.tun.get_session().into())?;
        Ok(AsyncDevice {
            inner: device,
            session_reader,
            session_writer,
        })
    }

    /// Consumes this AsyncDevice and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let mtu = self.mtu().unwrap_or(crate::DEFAULT_MTU);
        let codec = TunPacketCodec::new(mtu as usize);
        // guarantee to avoid the mtu of wintun may far away larger than the default provided capacity of ReadBuf of Framed
        Framed::with_capacity(self, codec, mtu as usize)
    }

    pub fn split(self) -> std::io::Result<(DeviceWriter, DeviceReader)> {
        Ok((self.session_writer, self.session_reader))
    }

    /// Recv a packet from tun device - Not implemented for windows
    pub async fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.session_reader.session.recv(buf).await
    }

    /// Send a packet to tun device - Not implemented for windows
    pub async fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.session_writer.session.send(buf).await
    }
}

impl AsyncRead for AsyncDevice {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.session_reader).poll_read(cx, buf)
    }
}

impl AsyncWrite for AsyncDevice {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.session_writer).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.session_writer).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.session_writer).poll_shutdown(cx)
    }
}

pub struct DeviceReader {
    session: AsyncSession,
}

impl DeviceReader {
    fn new(session: AsyncSession) -> std::io::Result<DeviceReader> {
        Ok(DeviceReader { session })
    }
}

impl AsyncRead for DeviceReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let buf_ref = buf.initialize_unfilled();
        match futures::AsyncRead::poll_read(Pin::new(&mut self.session), cx, buf_ref) {
            Poll::Ready(Ok(size)) => {
                buf.advance(size);
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct DeviceWriter {
    session: AsyncSession,
}

impl DeviceWriter {
    fn new(session: AsyncSession) -> std::io::Result<DeviceWriter> {
        Ok(DeviceWriter { session })
    }
}

impl AsyncWrite for DeviceWriter {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        futures::AsyncWrite::poll_write(Pin::new(&mut self.session), cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        futures::AsyncWrite::poll_flush(Pin::new(&mut self.session), cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        futures::AsyncWrite::poll_close(Pin::new(&mut self.session), cx)
    }
}
