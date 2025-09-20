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

use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use crate::Layer;
use crate::configuration::Configuration;
use crate::device::AbstractDevice;
use crate::error::{Error, Result};
use crate::run_command::run_command;
use crate::windows::AbstractDeviceExt;
use wintun_bindings::{Adapter, MAX_RING_CAPACITY, Session, load_from_path};

/// A TUN device using the wintun driver.
pub struct Device {
    pub(crate) tun: Tun,
    mtu: u16,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let layer = config.layer.unwrap_or(Layer::L3);
        if layer == Layer::L3 {
            let wintun_file = &config.platform_config.wintun_file;
            let wintun = unsafe { load_from_path(wintun_file)? };
            let tun_name = config.tun_name.as_deref().unwrap_or("wintun");
            let guid = config.platform_config.device_guid;
            let adapter = match Adapter::open(&wintun, tun_name) {
                Ok(a) => a,
                Err(e) => {
                    log::debug!("failed to open adapter: {e}");
                    Adapter::create(&wintun, tun_name, tun_name, guid)?
                }
            };
            if let Some(metric) = config.metric {
                // command: netsh interface ip set interface {index} metric={metric}
                let i = adapter.get_adapter_index()?.to_string();
                let m = format!("metric={metric}");
                run_command("netsh", &["interface", "ip", "set", "interface", &i, &m])?;
            }
            let address = config
                .address
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)));
            let mask = config
                .netmask
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)));
            let gateway = config.destination;
            adapter.set_network_addresses_tuple(address, mask, gateway)?;
            if let Some(dns_servers) = &config.platform_config.dns_servers {
                adapter.set_dns_servers(dns_servers)?;
            }
            if let Some(mtu) = config.mtu {
                adapter.set_mtu(mtu as _)?;
            }
            let capacity = config.ring_capacity.unwrap_or(MAX_RING_CAPACITY);
            let session = adapter.start_session(capacity)?;
            let mut device = Device {
                tun: Tun { session },
                mtu: adapter.get_mtu()? as u16,
            };

            // This is not needed since we use netsh to set the address.
            device.configure(config)?;

            Ok(device)
        } else if layer == Layer::L2 {
            todo!()
        } else {
            panic!("unknow layer {layer:?}");
        }
    }

    pub fn split(self) -> (Reader, Writer) {
        let tun = Arc::new(self.tun);
        (Reader(tun.clone()), Writer(tun))
    }

    /// Recv a packet from tun device
    pub fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tun.recv(buf)
    }

    /// Send a packet to tun device
    pub fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.tun.send(buf)
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tun.read(buf)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tun.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tun.flush()
    }
}

impl AbstractDevice for Device {
    fn tun_index(&self) -> Result<i32> {
        Ok(self.tun.session.get_adapter().get_adapter_index()? as i32)
    }

    fn tun_name(&self) -> Result<String> {
        Ok(self.tun.session.get_adapter().get_name()?)
    }

    fn set_tun_name(&mut self, value: &str) -> Result<()> {
        Ok(self.tun.session.get_adapter().set_name(value)?)
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
        Ok(self.tun.session.get_adapter().set_address(value)?)
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
        Ok(self.tun.session.get_adapter().set_gateway(Some(value))?)
    }

    fn broadcast(&self) -> Result<IpAddr> {
        Err(Error::NotImplemented)
    }

    fn set_broadcast(&mut self, value: IpAddr) -> Result<()> {
        log::debug!("set_broadcast {value} is not need");
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
        Ok(self.tun.session.get_adapter().set_netmask(value)?)
    }

    /// The return value is always `Ok(65535)` due to wintun
    fn mtu(&self) -> Result<u16> {
        Ok(self.mtu)
    }

    /// This setting has no effect since the mtu of wintun is always 65535
    fn set_mtu(&mut self, mtu: u16) -> Result<()> {
        self.tun.session.get_adapter().set_mtu(mtu as _)?;
        self.mtu = mtu;
        Ok(())
    }

    fn packet_information(&self) -> bool {
        // Note: wintun does not support packet information
        false
    }
}

impl AbstractDeviceExt for Device {
    fn tun_luid(&self) -> u64 {
        // SAFETY: LUID is always a u64
        unsafe { self.tun.session.get_adapter().get_luid().Value }
    }
}

pub struct Tun {
    session: Arc<Session>,
}

impl Tun {
    pub fn get_session(&self) -> Arc<Session> {
        self.session.clone()
    }
    fn read_by_ref(&self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        use std::io::{Error, ErrorKind::ConnectionAborted};
        match self.session.receive_blocking() {
            Ok(pkt) => match std::io::copy(&mut pkt.bytes(), &mut buf) {
                Ok(n) => Ok(n as usize),
                Err(e) => Err(e),
            },
            Err(e) => Err(Error::new(ConnectionAborted, e)),
        }
    }
    fn write_by_ref(&self, mut buf: &[u8]) -> std::io::Result<usize> {
        use std::io::{Error, ErrorKind::OutOfMemory};
        let size = buf.len();
        match self.session.allocate_send_packet(size as u16) {
            Err(e) => Err(Error::new(OutOfMemory, e)),
            Ok(mut packet) => match std::io::copy(&mut buf, &mut packet.bytes_mut()) {
                Ok(s) => {
                    self.session.send_packet(packet);
                    Ok(s as usize)
                }
                Err(e) => Err(e),
            },
        }
    }

    /// Recv a packet from tun device
    pub fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_by_ref(buf)
    }

    /// Send a packet to tun device
    pub fn send(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_by_ref(buf)
    }
}

impl Read for Tun {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_by_ref(buf)
    }
}

impl Write for Tun {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_by_ref(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

// impl Drop for Tun {
//     fn drop(&mut self) {
//         // The session has implemented drop
//         if let Err(err) = self.session.shutdown() {
//             log::error!("failed to shutdown session: {:?}", err);
//         }
//     }
// }

#[repr(transparent)]
pub struct Reader(Arc<Tun>);

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read_by_ref(buf)
    }
}

#[repr(transparent)]
pub struct Writer(Arc<Tun>);

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write_by_ref(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
