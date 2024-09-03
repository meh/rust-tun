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

use crate::configuration::Configuration;
use crate::device::AbstractDevice;
use crate::error::{Error, Result};
use crate::run_command::run_command;
use crate::Layer;
use wintun_bindings::{load_from_path, Adapter, Session, MAX_RING_CAPACITY};

pub enum Driver {
    Tun(Tun),
    #[allow(dead_code)]
    Tap(()),
}
/// A TUN device using the wintun driver.
pub struct Device {
    pub(crate) driver: Driver,
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
                Err(_) => Adapter::create(&wintun, tun_name, tun_name, guid)?,
            };
            if let Some(metric) = config.metric {
                // Command: netsh interface ip set interface {index} metric={metric}
                let i = adapter.get_adapter_index()?.to_string();
                let m = format!("metric={}", metric);
                run_command("netsh", &["interface", "ip", "set", "interface", &i, &m])?;
            }
            let address = config
                .address
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 1, 0, 2)));
            let mask = config
                .netmask
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)));
            let gateway = config.destination.map(IpAddr::from);
            adapter.set_network_addresses_tuple(address, mask, gateway)?;
            #[cfg(feature = "wintun-dns")]
            if let Some(dns_servers) = &config.platform_config.dns_servers {
                adapter.set_dns_servers(dns_servers)?;
            }
            let mtu = config.mtu.unwrap_or(crate::DEFAULT_MTU);

            let capacity = config.ring_capacity.unwrap_or(MAX_RING_CAPACITY);
            let session = adapter.start_session(capacity)?;
            adapter.set_mtu(mtu as _)?;
            let mut device = Device {
                driver: Driver::Tun(Tun { session }),
                mtu,
            };

            // This is not needed since we use netsh to set the address.
            device.configure(config)?;

            Ok(device)
        } else if layer == Layer::L2 {
            todo!()
        } else {
            panic!("unknow layer {:?}", layer);
        }
    }

    pub fn split(self) -> (Reader, Writer) {
        match self.driver {
            Driver::Tun(tun) => {
                let tun = Arc::new(tun);
                (Reader(tun.clone()), Writer(tun))
            }
            Driver::Tap(_) => {
                unimplemented!()
            }
        }
    }

    /// Recv a packet from tun device
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        match &self.driver {
            Driver::Tun(tun) => tun.recv(buf),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    /// Send a packet to tun device
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        match &self.driver {
            Driver::Tun(tun) => tun.send(buf),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match &mut self.driver {
            Driver::Tun(tun) => tun.read(buf),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.driver {
            Driver::Tun(tun) => tun.write(buf),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.driver {
            Driver::Tun(tun) => tun.flush(),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }
}

impl AbstractDevice for Device {
    fn tun_name(&self) -> Result<String> {
        match &self.driver {
            Driver::Tun(tun) => Ok(tun.session.get_adapter().get_name()?),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn set_tun_name(&mut self, value: &str) -> Result<()> {
        match &self.driver {
            Driver::Tun(tun) => {
                tun.session.get_adapter().set_name(value)?;
                Ok(())
            }
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn enabled(&mut self, _value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<IpAddr> {
        match &self.driver {
            Driver::Tun(tun) => {
                let addresses = tun.session.get_adapter().get_addresses()?;
                addresses
                    .iter()
                    .find_map(|a| match a {
                        std::net::IpAddr::V4(a) => Some(std::net::IpAddr::V4(*a)),
                        _ => None,
                    })
                    .ok_or(Error::InvalidConfig)
            }
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn set_address(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        match &self.driver {
            Driver::Tun(tun) => {
                tun.session.get_adapter().set_address(value)?;
                Ok(())
            }
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn destination(&self) -> Result<IpAddr> {
        // It's just the default gateway in windows.
        match &self.driver {
            Driver::Tun(tun) => tun
                .session
                .get_adapter()
                .get_gateways()?
                .iter()
                .find_map(|a| match a {
                    std::net::IpAddr::V4(a) => Some(std::net::IpAddr::V4(*a)),
                    _ => None,
                })
                .ok_or(Error::InvalidConfig),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn set_destination(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        // It's just set the default gateway in windows.
        match &self.driver {
            Driver::Tun(tun) => {
                tun.session.get_adapter().set_gateway(Some(value))?;
                Ok(())
            }
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
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
        match &self.driver {
            Driver::Tun(tun) => tun
                .session
                .get_adapter()
                .get_netmask_of_address(&current_addr)
                .map_err(Error::WintunError),
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn set_netmask(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        match &self.driver {
            Driver::Tun(tun) => {
                tun.session.get_adapter().set_netmask(value)?;
                Ok(())
            }
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    /// The return value is always `Ok(65535)` due to wintun
    fn mtu(&self) -> Result<u16> {
        Ok(self.mtu)
    }

    /// This setting has no effect since the mtu of wintun is always 65535
    fn set_mtu(&mut self, mtu: u16) -> Result<()> {
        match &self.driver {
            Driver::Tun(tun) => {
                tun.session.get_adapter().set_mtu(mtu as _)?;
                self.mtu = mtu;
                Ok(())
            }
            Driver::Tap(_tap) => {
                unimplemented!()
            }
        }
    }

    fn packet_information(&self) -> bool {
        // Note: wintun does not support packet information
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

    /// Recv a packet from tun device
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_by_ref(buf)
    }

    /// Send a packet to tun device
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.write_by_ref(buf)
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
