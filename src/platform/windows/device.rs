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

use wintun::Session;

use crate::configuration::Configuration;
use crate::device::AbstractDevice;
use crate::error::{Error, Result};

/// A TUN device using the wintun driver.
pub struct Device {
    pub(crate) queue: Queue,
    mtu: usize,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let wintun = unsafe { wintun::load()? };
        let tun_name = config.name.as_deref().unwrap_or("wintun");
        let guid = config.platform_config.device_guid;
        let adapter = match wintun::Adapter::open(&wintun, tun_name) {
            Ok(a) => a,
            Err(_) => wintun::Adapter::create(&wintun, tun_name, tun_name, guid)?,
        };

        let address = config.address.unwrap_or(Ipv4Addr::new(10, 1, 0, 2));
        let mask = config.netmask.unwrap_or(Ipv4Addr::new(255, 255, 255, 0));
        let gateway = config.destination.map(IpAddr::from);
        adapter.set_network_addresses_tuple(IpAddr::V4(address), IpAddr::V4(mask), gateway)?;
        let mtu = config.mtu.unwrap_or(u16::MAX as i32) as usize;

        let session = adapter.start_session(wintun::MAX_RING_CAPACITY)?;

        let mut device = Device {
            queue: Queue {
                session: Arc::new(session),
            },
            mtu,
        };

        // This is not needed since we use netsh to set the address.
        device.configure(config)?;

        Ok(device)
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queue.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.queue.read_vectored(bufs)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.queue.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.queue.write_vectored(bufs)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.queue.flush()
    }
}

impl AbstractDevice for Device {
    type Queue = Queue;

    fn name(&self) -> Result<String> {
        Ok(self.queue.session.get_adapter().get_name()?)
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        self.queue.session.get_adapter().set_name(value)?;
        Ok(())
    }

    fn enabled(&mut self, _value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<Ipv4Addr> {
        let addresses = self.queue.session.get_adapter().get_addresses()?;
        addresses
            .iter()
            .find_map(|a| match a {
                std::net::IpAddr::V4(a) => Some(*a),
                _ => None,
            })
            .ok_or(Error::InvalidConfig)
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        self.queue.session.get_adapter().set_address(value)?;
        Ok(())
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        // It's just the default gateway in windows.
        self.queue
            .session
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
        self.queue.session.get_adapter().set_gateway(Some(value))?;
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
            .queue
            .session
            .get_adapter()
            .get_netmask_of_address(&IpAddr::V4(current_addr))?;
        match netmask {
            IpAddr::V4(netmask) => Ok(netmask),
            _ => Err(Error::InvalidConfig),
        }
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        self.queue.session.get_adapter().set_netmask(value)?;
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
        Some(&mut self.queue)
    }
}

pub struct Queue {
    session: Arc<Session>,
}

impl Queue {
    pub fn get_session(&self) -> Arc<Session> {
        self.session.clone()
    }
}

impl Read for Queue {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        let reader_session = self.session.clone();
        match reader_session.receive_blocking() {
            Ok(pkt) => match io::copy(&mut pkt.bytes(), &mut buf) {
                Ok(n) => Ok(n as usize),
                Err(e) => Err(e),
            },
            Err(e) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, e)),
        }
    }
}

impl Write for Queue {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        let size = buf.len();
        match self.session.allocate_send_packet(size as u16) {
            Err(e) => Err(io::Error::new(io::ErrorKind::OutOfMemory, e)),
            Ok(mut packet) => match io::copy(&mut buf, &mut packet.bytes_mut()) {
                Ok(s) => {
                    self.session.send_packet(packet);
                    Ok(s as usize)
                }
                Err(e) => Err(e),
            },
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        if let Err(err) = self.session.shutdown() {
            log::error!("failed to shutdown session: {:?}", err);
        }
    }
}
