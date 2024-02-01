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
use std::io;
use std::io::Error;

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::codec::Framed;

use super::TunPacketCodec;
use crate::device::AbstractDevice;
use crate::platform::Device;

pub struct AsyncDevice {
    inner: Device,
    session: WinSession,
}

/// Returns a shared reference to the underlying Device object
impl AsRef<Device> for AsyncDevice {
    fn as_ref(&self) -> &Device {
        &self.inner
    }
}

/// Returns a mutable reference to the underlying Device object
impl AsMut<Device> for AsyncDevice {
    fn as_mut(&mut self) -> &mut Device {
        &mut self.inner
    }
}

impl AsyncDevice {
    /// Create a new `AsyncDevice` wrapping around a `Device`.
    pub fn new(device: Device) -> io::Result<AsyncDevice> {
        let session = WinSession::new(device.tun.get_session())?;
        Ok(AsyncDevice {
            inner: device,
            session,
        })
    }

    /// Consumes this AsyncDevice and return a Framed object (unified Stream and Sink interface)
    pub fn into_framed(self) -> Framed<Self, TunPacketCodec> {
        let mtu = self.as_ref().mtu().unwrap_or(crate::DEFAULT_MTU);
        let codec = TunPacketCodec::new(mtu as usize);
        // guarantee to avoid the mtu of wintun may far away larger than the default provided capacity of ReadBuf of Framed
        Framed::with_capacity(self, codec, mtu as usize)
    }
}

impl AsyncRead for AsyncDevice {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.session).poll_read(cx, buf)
    }
}

impl AsyncWrite for AsyncDevice {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        Pin::new(&mut self.session).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.session).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Pin::new(&mut self.session).poll_shutdown(cx)
    }
}

struct WinSession {
    session: std::sync::Arc<wintun::Session>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    _task: std::thread::JoinHandle<()>,
}

impl WinSession {
    fn new(session: std::sync::Arc<wintun::Session>) -> Result<WinSession, io::Error> {
        let session_reader = session.clone();
        let (receiver_tx, receiver_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        let task = std::thread::spawn(move || loop {
            match session_reader.receive_blocking() {
                Ok(packet) => {
                    if let Err(err) = receiver_tx.send(packet.bytes().to_vec()) {
                        log::error!("{}", err);
                    }
                }
                Err(err) => {
                    log::info!("{}", err);
                    break;
                }
            }
        });

        Ok(WinSession {
            session,
            receiver: receiver_rx,
            _task: task,
        })
    }
}

impl AsyncRead for WinSession {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match std::task::ready!(self.receiver.poll_recv(cx)) {
            Some(bytes) => {
                buf.put_slice(&bytes);
                std::task::Poll::Ready(Ok(()))
            }
            None => std::task::Poll::Ready(Ok(())),
        }
    }
}

impl AsyncWrite for WinSession {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let mut write_pack = self.session.allocate_send_packet(buf.len() as u16)?;
        write_pack.bytes_mut().copy_from_slice(buf.as_ref());
        self.session.send_packet(write_pack);
        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Poll::Ready(Ok(()))
    }
}
