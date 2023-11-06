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

use libc::{
    self, c_char, c_short, ifreq, AF_INET, IFF_MULTI_QUEUE, IFF_NO_PI, IFF_RUNNING, IFF_TAP,
    IFF_TUN, IFF_UP, IFNAMSIZ, O_RDWR, SOCK_DGRAM,
};
use std::{
    ffi::{CStr, CString},
    io::{self, Read, Write},
    mem,
    net::Ipv4Addr,
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    ptr,
    sync::Arc,
    vec::Vec,
};

use crate::{
    configuration::{Configuration, Layer},
    device::Device as D,
    error::*,
    platform::linux::sys::*,
    platform::posix::{self, Fd, SockAddr},
};

/// A TUN device using the TUN/TAP Linux driver.
pub struct Device {
    name: String,
    queues: Vec<Queue>,
    ctl: Fd,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let mut device = unsafe {
            let dev = match config.name.as_ref() {
                Some(name) => {
                    let name = CString::new(name.clone())?;

                    if name.as_bytes_with_nul().len() > IFNAMSIZ {
                        return Err(Error::NameTooLong);
                    }

                    Some(name)
                }

                None => None,
            };

            let mut queues = Vec::new();

            let mut req: ifreq = mem::zeroed();

            if let Some(dev) = dev.as_ref() {
                ptr::copy_nonoverlapping(
                    dev.as_ptr() as *const c_char,
                    req.ifr_name.as_mut_ptr(),
                    dev.as_bytes().len(),
                );
            }

            let device_type: c_short = config.layer.unwrap_or(Layer::L3).into();

            let queues_num = config.queues.unwrap_or(1);
            if queues_num < 1 {
                return Err(Error::InvalidQueuesNumber);
            }

            let iff_no_pi = IFF_NO_PI as c_short;
            let iff_multi_queue = IFF_MULTI_QUEUE as c_short;
            let packet_information = config.platform.packet_information;
            req.ifr_ifru.ifru_flags = device_type
                | if packet_information { 0 } else { iff_no_pi }
                | if queues_num > 1 { iff_multi_queue } else { 0 };

            for _ in 0..queues_num {
                let tun = Fd::new(libc::open(b"/dev/net/tun\0".as_ptr() as *const _, O_RDWR))
                    .map_err(|_| io::Error::last_os_error())?;

                if tunsetiff(tun.0, &mut req as *mut _ as *mut _) < 0 {
                    return Err(io::Error::last_os_error().into());
                }

                queues.push(Queue {
                    tun,
                    pi_enabled: config.platform.packet_information,
                });
            }

            let ctl = Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0))?;

            let name = CStr::from_ptr(req.ifr_name.as_ptr())
                .to_string_lossy()
                .to_string();
            Device { name, queues, ctl }
        };

        device.configure(config)?;

        Ok(device)
    }

    /// Prepare a new request.
    unsafe fn request(&self) -> ifreq {
        let mut req: ifreq = mem::zeroed();
        ptr::copy_nonoverlapping(
            self.name.as_ptr() as *const c_char,
            req.ifr_name.as_mut_ptr(),
            self.name.len(),
        );

        req
    }

    /// Make the device persistent.
    pub fn persist(&mut self) -> Result<()> {
        unsafe {
            if tunsetpersist(self.as_raw_fd(), &1) < 0 {
                Err(io::Error::last_os_error().into())
            } else {
                Ok(())
            }
        }
    }

    /// Set the owner of the device.
    pub fn user(&mut self, value: i32) -> Result<()> {
        unsafe {
            if tunsetowner(self.as_raw_fd(), &value) < 0 {
                Err(io::Error::last_os_error().into())
            } else {
                Ok(())
            }
        }
    }

    /// Set the group of the device.
    pub fn group(&mut self, value: i32) -> Result<()> {
        unsafe {
            if tunsetgroup(self.as_raw_fd(), &value) < 0 {
                Err(io::Error::last_os_error().into())
            } else {
                Ok(())
            }
        }
    }

    /// Return whether the device has packet information
    pub fn has_packet_information(&mut self) -> bool {
        self.queues[0].has_packet_information()
    }

    /// Split the interface into a `Reader` and `Writer`.
    pub fn split(mut self) -> (posix::Reader, posix::Writer) {
        let fd = Arc::new(self.queues.swap_remove(0).tun);
        (posix::Reader(fd.clone()), posix::Writer(fd.clone()))
    }

    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.queues[0].set_nonblock()
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.queues[0].read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [io::IoSliceMut<'_>]) -> io::Result<usize> {
        self.queues[0].read_vectored(bufs)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.queues[0].write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.queues[0].flush()
    }

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        self.queues[0].write_vectored(bufs)
    }
}

impl D for Device {
    type Queue = Queue;

    fn name(&self) -> Result<String> {
        Ok(self.name.clone())
    }

    fn set_name(&mut self, value: &str) -> Result<()> {
        unsafe {
            let name = CString::new(value)?;

            if name.as_bytes_with_nul().len() > IFNAMSIZ {
                return Err(Error::NameTooLong);
            }

            let mut req = self.request();
            ptr::copy_nonoverlapping(
                name.as_ptr() as *const c_char,
                req.ifr_ifru.ifru_newname.as_mut_ptr(),
                value.len(),
            );

            if siocsifname(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            self.name = value.into();

            Ok(())
        }
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        unsafe {
            let mut req = self.request();

            if siocgifflags(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            if value {
                req.ifr_ifru.ifru_flags |= (IFF_UP | IFF_RUNNING) as c_short;
            } else {
                req.ifr_ifru.ifru_flags &= !(IFF_UP as c_short);
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

            SockAddr::new(&req.ifr_ifru.ifru_addr).map(Into::into)
        }
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_addr = SockAddr::from(value).into();

            if siocsifaddr(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        unsafe {
            let mut req = self.request();

            if siocgifdstaddr(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifr_ifru.ifru_dstaddr).map(Into::into)
        }
    }

    fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_dstaddr = SockAddr::from(value).into();

            if siocsifdstaddr(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn broadcast(&self) -> Result<Ipv4Addr> {
        unsafe {
            let mut req = self.request();

            if siocgifbrdaddr(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifr_ifru.ifru_broadaddr).map(Into::into)
        }
    }

    fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_broadaddr = SockAddr::from(value).into();

            if siocsifbrdaddr(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn netmask(&self) -> Result<Ipv4Addr> {
        unsafe {
            let mut req = self.request();

            if siocgifnetmask(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifr_ifru.ifru_netmask).map(Into::into)
        }
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_netmask = SockAddr::from(value).into();

            if siocsifnetmask(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn mtu(&self) -> Result<i32> {
        unsafe {
            let mut req = self.request();

            if siocgifmtu(self.ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(req.ifr_ifru.ifru_mtu)
        }
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_mtu = value;

            if siocsifmtu(self.ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn queue(&mut self, index: usize) -> Option<&mut Self::Queue> {
        self.queues.get_mut(index)
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.queues[0].as_raw_fd()
    }
}

impl IntoRawFd for Device {
    fn into_raw_fd(mut self) -> RawFd {
        // It is Ok to swap the first queue with the last one, because the self will be dropped afterwards
        let queue = self.queues.swap_remove(0);
        queue.into_raw_fd()
    }
}

pub struct Queue {
    tun: Fd,
    pi_enabled: bool,
}

impl Queue {
    pub fn has_packet_information(&mut self) -> bool {
        self.pi_enabled
    }

    pub fn set_nonblock(&self) -> io::Result<()> {
        self.tun.set_nonblock()
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

impl From<Layer> for c_short {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::L2 => IFF_TAP as c_short,
            Layer::L3 => IFF_TUN as c_short,
        }
    }
}
