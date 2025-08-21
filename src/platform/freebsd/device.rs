//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, March 2024
//
// Copyleft (ↄ) xmh. <970252187@qq.com>
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
    self, AF_INET, IFF_RUNNING, IFF_UP, IFNAMSIZ, O_RDWR, SOCK_DGRAM, c_char, c_short, ifreq,
};
use std::{
    // ffi::{CStr, CString},
    io::{Read, Write},
    mem,
    net::{IpAddr, Ipv4Addr},
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    ptr,
};

use crate::{
    configuration::{Configuration, Layer},
    device::AbstractDevice,
    error::{Error, Result},
    platform::freebsd::sys::*,
    platform::posix::{self, Fd, Tun, sockaddr_union},
    run_command::run_command,
};

#[derive(Clone, Copy)]
struct Route {
    addr: Ipv4Addr,
    netmask: Ipv4Addr,
    dest: Ipv4Addr,
}

/// A TUN device using the TUN/TAP Linux driver.
pub struct Device {
    tun_name: String,
    pub(crate) tun: Tun,
    ctl: Fd,
    route: Option<Route>,
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

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let mut device = unsafe {
            let dev = match config.tun_name.as_ref() {
                Some(tun_name) => {
                    let tun_name = tun_name.clone();

                    if tun_name.len() > IFNAMSIZ {
                        return Err(Error::NameTooLong);
                    }

                    Some(tun_name)
                }

                None => None,
            };

            if config.layer.filter(|l| *l != Layer::L3).is_some() {
                return Err(Error::UnsupportedLayer);
            }

            let queues_num = config.queues.unwrap_or(1);
            if queues_num != 1 {
                return Err(Error::InvalidQueuesNumber);
            }

            let ctl = Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0), true)?;

            let (tun, tun_name) = {
                if let Some(name) = dev.as_ref() {
                    let device_path = format!("/dev/{name}\0");
                    let fd = libc::open(device_path.as_ptr() as *const _, O_RDWR);
                    let tun = Fd::new(fd, true).map_err(|_| std::io::Error::last_os_error())?;
                    (tun, name.clone())
                } else {
                    let (tun, device_name) = 'End: {
                        for i in 0..256 {
                            let device_name = format!("tun{i}");
                            let device_path = format!("/dev/{device_name}\0");
                            let fd = libc::open(device_path.as_ptr() as *const _, O_RDWR);
                            if fd > 0 {
                                use std::io::Error;
                                let tun = Fd::new(fd, true).map_err(|_| Error::last_os_error())?;
                                break 'End (tun, device_name);
                            }
                        }
                        use std::io::ErrorKind::AlreadyExists;
                        let info = "no avaiable file descriptor";
                        return Err(Error::Io(std::io::Error::new(AlreadyExists, info)));
                    };
                    (tun, device_name)
                }
            };

            let mtu = config.mtu.unwrap_or(crate::DEFAULT_MTU);

            Device {
                tun_name,
                tun: Tun::new(tun, mtu, false),
                ctl,
                route: None,
            }
        };

        device.set_alias(
            config
                .address
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))),
            config
                .destination
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 255))),
            config
                .netmask
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0))),
        )?;

        device.configure(config)?;

        Ok(device)
    }

    /// Set the IPv4 alias of the device.
    fn set_alias(&mut self, addr: IpAddr, dest: IpAddr, mask: IpAddr) -> Result<()> {
        let IpAddr::V4(addr) = addr else {
            unimplemented!("do not support IPv6 yet")
        };
        let IpAddr::V4(dest) = dest else {
            unimplemented!("do not support IPv6 yet")
        };
        let IpAddr::V4(mask) = mask else {
            unimplemented!("do not support IPv6 yet")
        };
        let ctl = &self.ctl;
        unsafe {
            let mut req: ifaliasreq = mem::zeroed();
            ptr::copy_nonoverlapping(
                self.tun_name.as_ptr() as *const c_char,
                req.ifran.as_mut_ptr(),
                self.tun_name.len(),
            );

            req.addr = posix::sockaddr_union::from((addr, 0)).addr;
            req.dstaddr = posix::sockaddr_union::from((dest, 0)).addr;
            req.mask = posix::sockaddr_union::from((mask, 0)).addr;

            if let Err(err) = siocaifaddr(ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }

            let route = Route {
                addr,
                netmask: mask,
                dest,
            };
            if let Err(e) = self.set_route(route) {
                log::warn!("{e:?}");
            }

            Ok(())
        }
    }

    /// Prepare a new request.
    unsafe fn request(&self) -> ifreq {
        let mut req: ifreq = unsafe { mem::zeroed() };
        unsafe {
            ptr::copy_nonoverlapping(
                self.tun_name.as_ptr() as *const c_char,
                req.ifr_name.as_mut_ptr(),
                self.tun_name.len(),
            )
        };

        req
    }

    /// Split the interface into a `Reader` and `Writer`.
    pub fn split(self) -> (posix::Reader, posix::Writer) {
        (self.tun.reader, self.tun.writer)
    }

    /// Set non-blocking mode
    #[allow(dead_code)]
    pub(crate) fn set_nonblock(&self) -> std::io::Result<()> {
        self.tun.set_nonblock()
    }

    fn set_route(&mut self, route: Route) -> Result<()> {
        // if let Some(v) = &self.route {
        //     let prefix_len = ipnet::ip_mask_to_prefix(IpAddr::V4(v.netmask))
        //         .map_err(|_| Error::InvalidConfig)?;
        //     let network = ipnet::Ipv4Net::new(v.addr, prefix_len)
        //         .map_err(|e| Error::InvalidConfig)?
        //         .network();
        //     // command: route -n delete -net 10.0.0.0/24 10.0.0.1
        //     let args = [
        //         "-n",
        //         "delete",
        //         "-net",
        //         &format!("{}/{}", network, prefix_len),
        //         &v.dest.to_string(),
        //     ];
        // 	println!("{args:?}");
        //     run_command("route", &args);
        //     log::info!("route {}", args.join(" "));
        // }

        // command: route -n add -net 10.0.0.9/24 10.0.0.1
        let prefix_len = ipnet::ip_mask_to_prefix(IpAddr::V4(route.netmask))
            .map_err(|_| Error::InvalidConfig)?;
        let args = [
            "-n",
            "add",
            "-net",
            &format!("{}/{}", route.addr, prefix_len),
            &route.dest.to_string(),
        ];
        run_command("route", &args)?;
        log::info!("route {}", args.join(" "));
        self.route = Some(route);
        Ok(())
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

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.tun.read_vectored(bufs)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tun.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tun.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.tun.write_vectored(bufs)
    }
}

impl AbstractDevice for Device {
    fn tun_index(&self) -> Result<i32> {
        let name = self.tun_name()?;
        Ok(posix::tun_name_to_index(name)? as i32)
    }

    fn tun_name(&self) -> Result<String> {
        Ok(self.tun_name.clone())
    }

    fn set_tun_name(&mut self, value: &str) -> Result<()> {
        use std::ffi::CString;
        unsafe {
            if value.len() > IFNAMSIZ {
                return Err(Error::NameTooLong);
            }
            let mut req = self.request();
            let tun_name = CString::new(value)?;
            let mut tun_name: Vec<c_char> = tun_name
                .into_bytes_with_nul()
                .into_iter()
                .map(|c| c as c_char)
                .collect::<_>();
            req.ifr_ifru.ifru_data = tun_name.as_mut_ptr();
            if let Err(err) = siocsifname(self.ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }

            self.tun_name = value.to_string();
            Ok(())
        }
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        unsafe {
            let mut req = self.request();

            if let Err(err) = siocgifflags(self.ctl.as_raw_fd(), &mut req) {
                return Err(std::io::Error::from(err).into());
            }

            if value {
                req.ifr_ifru.ifru_flags[0] |= (IFF_UP | IFF_RUNNING) as c_short;
            } else {
                req.ifr_ifru.ifru_flags[0] &= !(IFF_UP as c_short);
            }

            if let Err(err) = siocsifflags(self.ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }

            Ok(())
        }
    }

    fn address(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifaddr(self.ctl.as_raw_fd(), &mut req) {
                return Err(std::io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_addr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_address(&mut self, value: IpAddr) -> Result<()> {
        unsafe {
            let req = self.request();
            if let Err(err) = siocdifaddr(self.ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }
            let previous = self.route.as_ref().ok_or(Error::InvalidConfig)?;
            self.set_alias(
                value,
                IpAddr::V4(previous.dest),
                IpAddr::V4(previous.netmask),
            )?;
        }
        Ok(())
    }

    fn destination(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifdstaddr(self.ctl.as_raw_fd(), &mut req) {
                return Err(std::io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_dstaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_destination(&mut self, value: IpAddr) -> Result<()> {
        unsafe {
            let req = self.request();
            if let Err(err) = siocdifaddr(self.ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }
            let previous = self.route.as_ref().ok_or(Error::InvalidConfig)?;
            self.set_alias(
                IpAddr::V4(previous.addr),
                value,
                IpAddr::V4(previous.netmask),
            )?;
        }
        Ok(())
    }

    fn broadcast(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifbrdaddr(self.ctl.as_raw_fd(), &mut req) {
                return Err(std::io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_broadaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_broadcast(&mut self, _value: IpAddr) -> Result<()> {
        Ok(())
    }

    fn netmask(&self) -> Result<IpAddr> {
        unsafe {
            let mut req = self.request();
            if let Err(err) = siocgifnetmask(self.ctl.as_raw_fd(), &mut req) {
                return Err(std::io::Error::from(err).into());
            }
            // NOTE: Here should be `ifru_netmask` instead of `ifru_addr`, but `ifreq` does not define it.
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_addr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_netmask(&mut self, value: IpAddr) -> Result<()> {
        unsafe {
            let req = self.request();
            if let Err(err) = siocdifaddr(self.ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }
            let previous = self.route.as_ref().ok_or(Error::InvalidConfig)?;
            self.set_alias(IpAddr::V4(previous.addr), IpAddr::V4(previous.dest), value)?;
        }
        Ok(())
    }

    fn mtu(&self) -> Result<u16> {
        unsafe {
            let mut req = self.request();

            if let Err(err) = siocgifmtu(self.ctl.as_raw_fd(), &mut req) {
                return Err(std::io::Error::from(err).into());
            }

            req.ifr_ifru
                .ifru_mtu
                .try_into()
                .map_err(|_| Error::TryFromIntError)
        }
    }

    fn set_mtu(&mut self, value: u16) -> Result<()> {
        unsafe {
            let mut req = self.request();
            req.ifr_ifru.ifru_mtu = value as i32;

            if let Err(err) = siocsifmtu(self.ctl.as_raw_fd(), &req) {
                return Err(std::io::Error::from(err).into());
            }
            self.tun.set_mtu(value);
            Ok(())
        }
    }

    fn set_routes(&mut self, _routes: &[crate::route::RouteEntry]) -> Result<()> {
        unimplemented!("freebsd routes coming soon...");
    }

    fn packet_information(&self) -> bool {
        self.tun.packet_information()
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.tun.as_raw_fd()
    }
}

impl IntoRawFd for Device {
    fn into_raw_fd(self) -> RawFd {
        self.tun.into_raw_fd()
    }
}

impl From<Layer> for c_short {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::L2 => 2,
            Layer::L3 => 3,
        }
    }
}
