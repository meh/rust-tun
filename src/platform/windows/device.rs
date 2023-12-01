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

use std::io::{self, Read, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::thread;
use std::vec::Vec;

use bytes::BufMut;
use tokio::io::ReadBuf;
use wintun::Session;

use crate::configuration::Configuration;
use crate::device::Device as D;
use crate::error::*;

/// A TUN device using the wintun driver.
pub struct Device {
    session: Arc<Session>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>,
    _task: thread::JoinHandle<()>,
    mtu:usize
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let wintun = unsafe { wintun::load()? };
        let tun_name = config.name.as_deref().unwrap_or("wintun");
        let guid = Some(9099482345783245345345_u128);
        let adapter = match wintun::Adapter::open(&wintun, tun_name) {
            Ok(a) => a,
            Err(_) => wintun::Adapter::create(&wintun, tun_name, tun_name, guid)?,
        };

        let address = config.address.ok_or(Error::InvalidConfig)?;
        let mask = config.netmask.ok_or(Error::InvalidConfig)?;
        let gateway = config.destination.map(IpAddr::from);
        adapter.set_network_addresses_tuple(IpAddr::V4(address), IpAddr::V4(mask), gateway)?;
        let mtu = config.mtu.unwrap_or(1500) as usize;

        let session = Arc::new(adapter.start_session(wintun::MAX_RING_CAPACITY)?);

        let (receiver_tx, receiver_rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

        let session_reader = session.clone();
        let task = thread::spawn(move || {
            loop {
                let packet = session_reader.receive_blocking().unwrap();
                let bytes = packet.bytes().to_vec();
                // dbg!(&bytes);
                receiver_tx.send(bytes).unwrap();
            }
        });

        let mut device = Device {
            session,
            receiver: receiver_rx,
            _task: task,
            mtu
        };

        // This is not needed since we use netsh to set the address.
        device.configure(config)?;

        Ok(device)
    }

    pub fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match std::task::ready!(self.receiver.poll_recv(cx)) {
            Some(bytes) => {
                buf.put_slice(&bytes[..]);
                std::task::Poll::Ready(Ok(()))
            }
            None => std::task::Poll::Ready(Ok(())),
        }
    }

    pub fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let mut write_pack = self.session.allocate_send_packet(buf.len() as u16)?;
        write_pack.bytes_mut().copy_from_slice(buf.as_ref());
        self.session.send_packet(write_pack);
        std::task::Poll::Ready(Ok(buf.len()))
    }

    pub fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    pub fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.receiver.blocking_recv(){
            Some(pkt) =>{
                buf.clone_from_slice(&pkt[..]);
                return Ok(pkt.len())
            },
            None => Ok(0),
        }
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        let mut write_pack = self.session.allocate_send_packet(len as u16)?;
        write_pack.bytes_mut().copy_from_slice(buf.as_ref());
        self.session.send_packet(write_pack);
        Ok(len)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl D for Device {
    type Queue = Device;

    fn name(&self) -> Result<String> {
        Ok(self.session.get_adapter().get_name()?)
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        self.session.get_adapter().set_name(value)?;
        Ok(())
    }

    fn enabled(&mut self, _value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<Ipv4Addr> {
        let addresses = self.session.get_adapter().get_addresses()?;
        addresses
            .iter()
            .find_map(|a| match a {
                std::net::IpAddr::V4(a) => Some(*a),
                _ => None,
            })
            .ok_or(Error::InvalidConfig)
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        self.session.get_adapter().set_address(value)?;
        Ok(())
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        // It's just the default gateway in windows.
        self.session
            .get_adapter()
            .get_gateways()?
            .iter()
            .find_map(|a| match a {
                std::net::IpAddr::V4(a) => Some(*a),
                _ => None,
            })
            .ok_or(Error::InvalidConfig)
    }

    fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
        // It's just set the default gateway in windows.
        self.session.get_adapter().set_gateway(Some(value))?;
        Ok(())
    }

    fn broadcast(&self) -> Result<Ipv4Addr> {
        Err(Error::NotImplemented)
    }

    fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
        log::debug!("set_broadcast {} is not need", value);
        Ok(())
    }

    fn netmask(&self) -> Result<Ipv4Addr> {
        let current_addr = self.address()?;
        let netmask = self
            .session
            .get_adapter()
            .get_netmask_of_address(&IpAddr::V4(current_addr))?;
        match netmask {
            IpAddr::V4(netmask) => Ok(netmask),
            _ => Err(Error::InvalidConfig),
        }
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        self.session.get_adapter().set_netmask(value)?;
        Ok(())
    }

    fn mtu(&self) -> Result<i32> {
        Ok(self.mtu as i32)
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        self.mtu = value as usize;
        Ok(())
    }

    fn queue(&mut self, _index: usize) -> Option<&mut Self::Queue> {
        Some(self)
    }
}

pub struct Queue;


