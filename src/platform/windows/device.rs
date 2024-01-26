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
    pub(crate) tun: Tun,
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

        let address = config
            .address
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)));
        let mask = config
            .netmask
            .unwrap_or(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)));
        let gateway = config.destination.map(IpAddr::from);
        adapter.set_network_addresses_tuple(address, mask, gateway)?;
        let mtu = config.mtu.unwrap_or(crate::DEFAULT_MTU);

        let session = adapter.start_session(wintun::MAX_RING_CAPACITY)?;

        let mut device = Device {
            tun: Tun {
                session: Arc::new(session),
            },
            mtu,
        };

        // This is not needed since we use netsh to set the address.
        device.configure(config)?;

        Ok(device)
    }

    pub fn split(self) -> (Reader, Writer) {
        let tun = Arc::new(self.tun);
        (Reader(tun.clone()), Writer(tun))
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.tun.read(buf)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tun.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tun.flush()
    }
}

impl AsRef<dyn AbstractDevice + 'static> for Device {
    fn as_ref(&self) -> &(dyn AbstractDevice + 'static) {
        self
    }
}

impl AsMut<dyn AbstractDevice + 'static> for Device {
    fn as_mut(&mut self) -> &mut (dyn AbstractDevice + 'static) {
        self
    }
}

impl AbstractDevice for Device {
    fn name(&self) -> Result<String> {
        Ok(self.tun.session.get_adapter().get_name()?)
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        self.tun.session.get_adapter().set_name(value)?;
        Ok(())
    }

    fn enabled(&mut self, _value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<IpAddr> {
        let addresses = self.tun.session.get_adapter().get_addresses()?;
        addresses
            .iter()
            .find_map(|a| match a {
                std::net::IpAddr::V4(a) => Some(std::net::IpAddr::V4(*a)),
                _ => None,
            })
            .ok_or(Error::InvalidConfig)
    }

    fn set_address(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        self.tun.session.get_adapter().set_address(value)?;
        Ok(())
    }

    fn destination(&self) -> Result<IpAddr> {
        // It's just the default gateway in windows.
        self.tun
            .session
            .get_adapter()
            .get_gateways()?
            .iter()
            .find_map(|a| match a {
                std::net::IpAddr::V4(a) => Some(std::net::IpAddr::V4(*a)),
                _ => None,
            })
            .ok_or(Error::InvalidConfig)
    }

    fn set_destination(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        // It's just set the default gateway in windows.
        self.tun.session.get_adapter().set_gateway(Some(value))?;
        Ok(())
    }

    fn broadcast(&self) -> Result<IpAddr> {
        Err(Error::NotImplemented)
    }

    fn set_broadcast(&mut self, value: IpAddr) -> Result<()> {
        log::debug!("set_broadcast {} is not need", value);
        Ok(())
    }

    fn netmask(&self) -> Result<IpAddr> {
        let current_addr = self.address()?;
        self.tun
            .session
            .get_adapter()
            .get_netmask_of_address(&current_addr)
            .map_err(Error::WintunError)
    }

    fn set_netmask(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        self.tun.session.get_adapter().set_netmask(value)?;
        Ok(())
    }

    fn mtu(&self) -> Result<usize> {
        // Note: wintun mtu is always 65535
        Ok(self.mtu)
    }

    fn set_mtu(&mut self, _: usize) -> Result<()> {
        // Note: no-op due to mtu of wintun is always 65535
        Ok(())
    }

    fn packet_information(&self) -> bool {
        false
    }
}

pub struct Tun {
    session: Arc<Session>,
}

impl Tun {
    pub fn get_session(&self) -> Arc<Session> {
        self.session.clone()
    }
    fn read_by_ref(&self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.session.receive_blocking() {
            Ok(pkt) => match io::copy(&mut pkt.bytes(), &mut buf) {
                Ok(n) => Ok(n as usize),
                Err(e) => Err(e),
            },
            Err(e) => Err(io::Error::new(io::ErrorKind::ConnectionAborted, e)),
        }
    }
    fn write_by_ref(&self, mut buf: &[u8]) -> io::Result<usize> {
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
}

impl Read for Tun {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_by_ref(buf)
    }
}

impl Write for Tun {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_by_ref(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Tun {
    fn drop(&mut self) {
        if let Err(err) = self.session.shutdown() {
            log::error!("failed to shutdown session: {:?}", err);
        }
    }
}

#[repr(transparent)]
pub struct Reader(Arc<Tun>);

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read_by_ref(buf)
    }
}

#[repr(transparent)]
pub struct Writer(Arc<Tun>);

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write_by_ref(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
