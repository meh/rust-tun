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
#![allow(unused_variables)]

use std::io::{self, Read, Write};
use std::mem;
use std::net::Ipv4Addr;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};
use std::ptr;
use std::sync::Arc;
use std::fs::OpenOptions;

use libc;
use libc::{c_char, AF_INET, SOCK_DGRAM};

use crate::configuration::{Configuration, Layer};
use crate::device::Device as D;
use crate::error::*;
use crate::platform::openbsd::sys::*;
use crate::platform::posix::{self, Fd, SockAddr};

/// A TUN device using the TUN OpenBSD driver.
pub struct Device {
    name: String,
    config: Configuration,
    ctl: Fd,
    queue: Queue
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let name = match config.name.as_ref() {
            Some(name) => name,
            None => "tun0"
        };

        if name.len() > IFNAMSIZ {
            return Err(Error::NameTooLong);
        }

        if name.starts_with("tun") {
            if let Some(layer) = config.layer {
                if layer == Layer::L2 {
                    return Err(Error::UnsupportedLayer);
                }
            }
        } else if name.starts_with("tap") {
            if let Some(layer) = config.layer {
                if layer == Layer::L3 {
                    return Err(Error::UnsupportedLayer);
                }
            }
        } else {
            return Err(Error::InvalidName);
        }

        let queues_number = config.queues.unwrap_or(1);
        if queues_number != 1 {
            return Err(Error::InvalidQueuesNumber);
        }

        let char_dev = OpenOptions::new().write(true)
            .read(true)
            .open(format!("/dev/{}", name))?;

        let ctl = Fd::new(unsafe {libc::socket(AF_INET, SOCK_DGRAM, 0)})
            .map_err(|_| io::Error::last_os_error())?;

        let queue = Queue { tun : Fd::new(char_dev.into_raw_fd())? };
        let mut device = Device {
            name: name.to_string(),
            config: config.clone(),
            ctl,
            queue,
        };

        device.configure(config)?;

        Ok(device)
    }

    /// Prepare a new request.
    pub unsafe fn request(&self) -> ifreq {
        let mut req: ifreq = mem::zeroed();
        ptr::copy_nonoverlapping(
            self.name.as_ptr() as *const c_char,
            req.name.as_mut_ptr(),
            self.name.len(),
        );

        req
    }

    /// Set the IPv4 alias of the device.
    pub fn set_alias(&mut self, addr: Ipv4Addr, mask: Ipv4Addr, destination: Option<Ipv4Addr>) -> Result<()> {
        unsafe {
            let mut req: ifaliasreq = mem::zeroed();
            ptr::copy_nonoverlapping(
                self.name.as_ptr() as *const c_char,
                req.name.as_mut_ptr(),
                self.name.len(),
            );

            req.addr = SockAddr::from(addr).into();
            req.mask = SockAddr::from(mask).into();
            if let Some(destination) = destination {
                req.dstaddr = SockAddr::from(destination).into();
            }

            if siocaifaddr(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    /// Split the interface into a `Reader` and `Writer`.
    pub fn split(self) -> (posix::Reader, posix::Writer) {
        let fd = Arc::new(self.queue.tun);
        (posix::Reader(fd.clone()), posix::Writer(fd.clone()))
    }

    /// Return whether the device has packet information
    pub fn has_packet_information(&self) -> bool {
        true
    }

    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.queue.set_nonblock()
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queue.tun.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.queue.tun.read_vectored(bufs)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.queue.tun.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.queue.tun.flush()
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.queue.tun.write_vectored(bufs)
    }
}

impl D for Device {
    type Queue = Queue;

    fn configure(&mut self, config: &Configuration) -> Result<()> {
        match (config.address, config.netmask, config.destination) {
            (Some(ip), Some(mask), dest) => {
                self.set_alias(ip, mask, dest)?
            },
            _ =>  {}
        }

        if let Some(mtu) = config.mtu {
            self.set_mtu(mtu)?;
            self.config.mtu = config.mtu;
        }

        if let Some(enabled) = config.enabled {
            self.enabled(enabled)?;
            self.config.enabled = config.enabled;
        }

        Ok(())
    }
    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        Err(Error::InvalidName)
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        unsafe {
            let mut req = self.request();

            if siocgifflags(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            if value {
                req.ifru.flags |= IFF_UP | IFF_RUNNING;
            } else {
                req.ifru.flags &= !IFF_UP;
            }

            if siocsifflags(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn address(&self) -> Result<Ipv4Addr> {
        unsafe {
            let mut req = self.request();

            if siocgifaddr(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifru.addr).map(Into::into)
        }
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        let netmask = self.config.netmask.ok_or(Error::InvalidAddress)?;
        let destination = self.config.destination;
        self.set_alias(value, netmask, destination)
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        unsafe {
            let mut req = self.request();

            if siocgifdstaddr(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifru.dstaddr).map(Into::into)
        }
    }

    fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
        let addr = self.config.address.ok_or(Error::InvalidAddress)?;
        let netmask = self.config.netmask.ok_or(Error::InvalidAddress)?;
        self.set_alias(addr, netmask, Some(value))
    }

    /// The same as getting destination on OpenBSD
    fn broadcast(&self) -> Result<Ipv4Addr> {
        self.destination()
    }

    /// The same as setting destination on OpenBSD
    fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
        self.set_destination(value)
    }

    fn netmask(&self) -> Result<Ipv4Addr> {
        unsafe {
            let mut req = self.request();

            if siocgifnetmask(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::unchecked(&req.ifru.addr).map(Into::into)
        }
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        let addr = self.config.address.ok_or(Error::InvalidAddress)?;
        let destination = self.config.destination;
        self.set_alias(addr, value, destination)
    }

    fn mtu(&self) -> Result<i32> {
        unsafe {
            let mut req = self.request();

            if siocgifmtu(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(req.ifru.metric)
        }
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifru.metric = value;

            if siocsifmtu(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn queue(&mut self, index: usize) -> Option<&mut Self::Queue> {
        if index > 0 {
            return None;
        }

        Some(&mut self.queue)
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.queue.as_raw_fd()
    }
}

impl IntoRawFd for Device {
    fn into_raw_fd(self) -> RawFd {
        self.queue.into_raw_fd()
    }
}

pub struct Queue {
    tun: Fd,
}

impl Queue {
    pub fn has_packet_information(&self) -> bool {
        // on openbsd this is always the case
        true
    }

    pub fn set_nonblock(&self) -> io::Result<()> {
        self.tun.set_nonblock()
    }
}

impl AsRawFd for Queue {
    fn as_raw_fd(&self) -> RawFd {
        self.tun.as_raw_fd()
    }
}

impl IntoRawFd for Queue {
    fn into_raw_fd(self) -> RawFd {
        self.tun.into_raw_fd()
    }
}

impl Read for Queue {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.tun.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.tun.read_vectored(bufs)
    }
}

impl Write for Queue {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.tun.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.tun.flush()
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.tun.write_vectored(bufs)
    }
}
