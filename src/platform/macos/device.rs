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

use crate::{
    configuration::{Configuration, Layer},
    device::Device as D,
    error::*,
    platform::{
        macos::sys::*,
        posix::{self, Fd, SockAddr},
    },
};
use libc::{
    self, c_char, c_short, c_uint, c_void, sockaddr, socklen_t, AF_INET, AF_SYSTEM, AF_SYS_CONTROL,
    IFF_RUNNING, IFF_UP, IFNAMSIZ, PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL, UTUN_OPT_IFNAME,
};
use std::{
    ffi::CStr,
    io::{self, Read, Write},
    mem,
    net::Ipv4Addr,
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    ptr,
    sync::Arc,
};

/// A TUN device using the TUN macOS driver.
pub struct Device {
    name: Option<String>,
    queue: Queue,
    ctl: Option<Fd>,
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        if let Some(fd) = config.raw_fd {
            let tun = Fd::new(fd).map_err(|_| io::Error::last_os_error())?;
            let device = Device {
                name: None,
                queue: Queue { tun },
                ctl: None,
            };
            return Ok(device);
        }

        let id = if let Some(name) = config.name.as_ref() {
            if name.len() > IFNAMSIZ {
                return Err(Error::NameTooLong);
            }

            if !name.starts_with("utun") {
                return Err(Error::InvalidName);
            }

            name[4..].parse::<u32>()? + 1_u32
        } else {
            0_u32
        };

        if config.layer.filter(|l| *l != Layer::L3).is_some() {
            return Err(Error::UnsupportedLayer);
        }

        let queues_number = config.queues.unwrap_or(1);
        if queues_number != 1 {
            return Err(Error::InvalidQueuesNumber);
        }

        let mut device = unsafe {
            let tun = Fd::new(libc::socket(PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL))?;

            let mut info = ctl_info {
                ctl_id: 0,
                ctl_name: {
                    let mut buffer = [0; 96];
                    for (i, o) in UTUN_CONTROL_NAME.as_bytes().iter().zip(buffer.iter_mut()) {
                        *o = *i as _;
                    }
                    buffer
                },
            };

            if ctliocginfo(tun.0, &mut info as *mut _ as *mut _) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            let addr = libc::sockaddr_ctl {
                sc_id: info.ctl_id,
                sc_len: mem::size_of::<libc::sockaddr_ctl>() as _,
                sc_family: AF_SYSTEM as _,
                ss_sysaddr: AF_SYS_CONTROL as _,
                sc_unit: id as c_uint,
                sc_reserved: [0; 5],
            };

            let address = &addr as *const libc::sockaddr_ctl as *const sockaddr;
            if libc::connect(tun.0, address, mem::size_of_val(&addr) as socklen_t) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            let mut name = [0u8; 64];
            let mut name_len: socklen_t = 64;

            let optval = &mut name as *mut _ as *mut c_void;
            let optlen = &mut name_len as *mut socklen_t;
            if libc::getsockopt(tun.0, SYSPROTO_CONTROL, UTUN_OPT_IFNAME, optval, optlen) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            let ctl = Some(Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0))?);

            Device {
                name: Some(
                    CStr::from_ptr(name.as_ptr() as *const c_char)
                        .to_string_lossy()
                        .into(),
                ),
                queue: Queue { tun },
                ctl,
            }
        };

        device.configure(config)?;
        device.set_alias(
            config.address.unwrap_or(Ipv4Addr::new(10, 0, 0, 1)),
            config.destination.unwrap_or(Ipv4Addr::new(10, 0, 0, 255)),
            config.netmask.unwrap_or(Ipv4Addr::new(255, 255, 255, 0)),
        )?;

        Ok(device)
    }

    /// Prepare a new request.
    /// # Safety
    pub unsafe fn request(&self) -> Result<libc::ifreq> {
        let name = self.name.as_ref().ok_or(Error::InvalidConfig)?;
        let mut req: libc::ifreq = mem::zeroed();
        ptr::copy_nonoverlapping(
            name.as_ptr() as *const c_char,
            req.ifr_name.as_mut_ptr(),
            name.len(),
        );

        Ok(req)
    }

    /// Set the IPv4 alias of the device.
    pub fn set_alias(&mut self, addr: Ipv4Addr, broadaddr: Ipv4Addr, mask: Ipv4Addr) -> Result<()> {
        let name = self.name.as_ref().ok_or(Error::InvalidConfig)?;
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req: ifaliasreq = mem::zeroed();
            ptr::copy_nonoverlapping(
                name.as_ptr() as *const c_char,
                req.ifran.as_mut_ptr(),
                name.len(),
            );

            req.addr = SockAddr::from(addr).into();
            req.broadaddr = SockAddr::from(broadaddr).into();
            req.mask = SockAddr::from(mask).into();

            if siocaifaddr(ctl.as_raw_fd(), &req) < 0 {
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
        self.queue.has_packet_information()
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

    fn name(&self) -> Result<String> {
        self.name.as_ref().cloned().ok_or(Error::InvalidConfig)
    }

    // XXX: Cannot set interface name on Darwin.
    fn set_name(&mut self, value: &str) -> Result<()> {
        Err(Error::InvalidName)
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if siocgifflags(ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            if value {
                req.ifr_ifru.ifru_flags |= (IFF_UP | IFF_RUNNING) as c_short;
            } else {
                req.ifr_ifru.ifru_flags &= !(IFF_UP as c_short);
            }

            if siocsifflags(ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn address(&self) -> Result<Ipv4Addr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if siocgifaddr(ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifr_ifru.ifru_addr).map(Into::into)
        }
    }

    fn set_address(&mut self, value: Ipv4Addr) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_addr = SockAddr::from(value).into();

            if siocsifaddr(ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn destination(&self) -> Result<Ipv4Addr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if siocgifdstaddr(ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifr_ifru.ifru_dstaddr).map(Into::into)
        }
    }

    fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_dstaddr = SockAddr::from(value).into();

            if siocsifdstaddr(ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn broadcast(&self) -> Result<Ipv4Addr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if siocgifbrdaddr(ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::new(&req.ifr_ifru.ifru_broadaddr).map(Into::into)
        }
    }

    fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_broadaddr = SockAddr::from(value).into();

            if siocsifbrdaddr(ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn netmask(&self) -> Result<Ipv4Addr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if siocgifnetmask(ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            SockAddr::unchecked(&req.ifr_ifru.ifru_addr).map(Into::into)
        }
    }

    fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_addr = SockAddr::from(value).into();

            if siocsifnetmask(ctl.as_raw_fd(), &req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(())
        }
    }

    fn mtu(&self) -> Result<i32> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if siocgifmtu(ctl.as_raw_fd(), &mut req) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            Ok(req.ifr_ifru.ifru_mtu)
        }
    }

    fn set_mtu(&mut self, value: i32) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_mtu = value;

            if siocsifmtu(ctl.as_raw_fd(), &req) < 0 {
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
        // on macos this is always the case
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
