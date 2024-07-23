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
    device::AbstractDevice,
    error::{Error, Result},
    platform::{
        macos::sys::*,
        posix::{self, ipaddr_to_sockaddr, sockaddr_union, Fd},
    },
};

const OVERWRITE_SIZE: usize = std::mem::size_of::<libc::__c_anonymous_ifr_ifru>();

use libc::{
    self, c_char, c_short, c_uint, c_void, sockaddr, socklen_t, AF_INET, AF_SYSTEM, AF_SYS_CONTROL,
    IFF_RUNNING, IFF_UP, IFNAMSIZ, PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL, UTUN_OPT_IFNAME,
};
use std::{
    ffi::CStr,
    io::{self, Read, Write},
    mem,
    net::{IpAddr, Ipv4Addr},
    os::unix::io::{AsRawFd, IntoRawFd, RawFd},
    ptr,
};

#[derive(Clone, Copy)]
struct Route {
    addr: Ipv4Addr,
    netmask: Ipv4Addr,
    dest: Ipv4Addr,
}

/// A TUN device using the TUN macOS driver.
pub struct Device {
    tun_name: Option<String>,
    tun: posix::Tun,
    ctl: Option<posix::Fd>,
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
        let mtu = config.mtu.unwrap_or(crate::DEFAULT_MTU);
        if let Some(fd) = config.raw_fd {
            let close_fd_on_drop = config.close_fd_on_drop.unwrap_or(true);
            let tun = Fd::new(fd, close_fd_on_drop).map_err(|_| io::Error::last_os_error())?;
            let device = Device {
                tun_name: None,
                tun: posix::Tun::new(tun, mtu, config.platform_config.packet_information),
                ctl: None,
                route: None,
            };
            return Ok(device);
        }

        let id = if let Some(tun_name) = config.tun_name.as_ref() {
            if tun_name.len() > IFNAMSIZ {
                return Err(Error::NameTooLong);
            }

            if !tun_name.starts_with("utun") {
                return Err(Error::InvalidName);
            }

            tun_name[4..].parse::<u32>()? + 1_u32
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
            let fd = libc::socket(PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL);
            let tun = posix::Fd::new(fd, true).map_err(|_| io::Error::last_os_error())?;

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

            if let Err(err) = ctliocginfo(tun.inner, &mut info as *mut _ as *mut _) {
                return Err(io::Error::from(err).into());
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
            if libc::connect(tun.inner, address, mem::size_of_val(&addr) as socklen_t) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            let mut tun_name = [0u8; 64];
            let mut name_len: socklen_t = 64;

            let optval = &mut tun_name as *mut _ as *mut c_void;
            let optlen = &mut name_len as *mut socklen_t;
            if libc::getsockopt(tun.inner, SYSPROTO_CONTROL, UTUN_OPT_IFNAME, optval, optlen) < 0 {
                return Err(io::Error::last_os_error().into());
            }

            let ctl = Some(posix::Fd::new(libc::socket(AF_INET, SOCK_DGRAM, 0), true)?);

            Device {
                tun_name: Some(
                    CStr::from_ptr(tun_name.as_ptr() as *const c_char)
                        .to_string_lossy()
                        .into(),
                ),
                tun: posix::Tun::new(tun, mtu, config.platform_config.packet_information),
                ctl,
                route: None,
            }
        };

        device.configure(config)?;
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

        Ok(device)
    }

    /// Prepare a new request.
    /// # Safety
    unsafe fn request(&self) -> Result<libc::ifreq> {
        let tun_name = self.tun_name.as_ref().ok_or(Error::InvalidConfig)?;
        let mut req: libc::ifreq = mem::zeroed();
        ptr::copy_nonoverlapping(
            tun_name.as_ptr() as *const c_char,
            req.ifr_name.as_mut_ptr(),
            tun_name.len(),
        );

        Ok(req)
    }

    /// Set the IPv4 alias of the device.
    fn set_alias(&mut self, addr: IpAddr, broadaddr: IpAddr, mask: IpAddr) -> Result<()> {
        let IpAddr::V4(addr) = addr else {
            unimplemented!("do not support IPv6 yet")
        };
        let IpAddr::V4(broadaddr) = broadaddr else {
            unimplemented!("do not support IPv6 yet")
        };
        let IpAddr::V4(mask) = mask else {
            unimplemented!("do not support IPv6 yet")
        };
        let tun_name = self.tun_name.as_ref().ok_or(Error::InvalidConfig)?;
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req: ifaliasreq = mem::zeroed();
            ptr::copy_nonoverlapping(
                tun_name.as_ptr() as *const c_char,
                req.ifra_name.as_mut_ptr(),
                tun_name.len(),
            );

            req.ifra_addr = sockaddr_union::from((addr, 0)).addr;
            req.ifra_broadaddr = sockaddr_union::from((broadaddr, 0)).addr;
            req.ifra_mask = sockaddr_union::from((mask, 0)).addr;

            if let Err(err) = siocaifaddr(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            let route = Route {
                addr,
                netmask: mask,
                dest: broadaddr,
            };
            if let Err(e) = self.set_route(route) {
                log::warn!("{e:?}");
            }
            Ok(())
        }
    }

    /// Split the interface into a `Reader` and `Writer`.
    pub fn split(self) -> (posix::Reader, posix::Writer) {
        (self.tun.reader, self.tun.writer)
    }

    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> io::Result<()> {
        self.tun.set_nonblock()
    }

    fn set_route(&mut self, route: Route) -> Result<()> {
        if let Some(v) = &self.route {
            let prefix_len = ipnet::ip_mask_to_prefix(IpAddr::V4(v.netmask))
                .map_err(|_| Error::InvalidConfig)?;
            let network = ipnet::Ipv4Net::new(v.addr, prefix_len)
                .map_err(|e| Error::InvalidConfig)?
                .network();
            // command: route -n delete -net 10.0.0.0/24 10.0.0.1
            let args = [
                "-n",
                "delete",
                "-net",
                &format!("{}/{}", network, prefix_len),
                &v.dest.to_string(),
            ];
            run_command("route", &args)?;
            log::info!("route {}", args.join(" "));
        }

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
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.tun.recv(buf)
    }

    /// Send a packet to tun device
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        self.tun.send(buf)
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

impl AbstractDevice for Device {
    fn tun_name(&self) -> Result<String> {
        self.tun_name.as_ref().cloned().ok_or(Error::InvalidConfig)
    }

    // XXX: Cannot set interface name on Darwin.
    fn set_tun_name(&mut self, value: &str) -> Result<()> {
        Err(Error::InvalidName)
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if let Err(err) = siocgifflags(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }

            if value {
                req.ifr_ifru.ifru_flags |= (IFF_UP | IFF_RUNNING) as c_short;
            } else {
                req.ifr_ifru.ifru_flags &= !(IFF_UP as c_short);
            }

            if let Err(err) = siocsifflags(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }

            Ok(())
        }
    }

    fn address(&self) -> Result<IpAddr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifaddr(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_addr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_address(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_addr, OVERWRITE_SIZE);
            if let Err(err) = siocsifaddr(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            if let Some(mut route) = self.route {
                route.addr = value;
                self.set_route(route)?;
            }
            Ok(())
        }
    }

    fn destination(&self) -> Result<IpAddr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifdstaddr(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_dstaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_destination(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_dstaddr, OVERWRITE_SIZE);
            if let Err(err) = siocsifdstaddr(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            if let Some(mut route) = self.route {
                route.dest = value;
                self.set_route(route)?;
            }
            Ok(())
        }
    }

    /// Question on macOS
    fn broadcast(&self) -> Result<IpAddr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifbrdaddr(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_broadaddr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    /// Question on macOS
    fn set_broadcast(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_broadaddr, OVERWRITE_SIZE);
            if let Err(err) = siocsifbrdaddr(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            Ok(())
        }
    }

    fn netmask(&self) -> Result<IpAddr> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            if let Err(err) = siocgifnetmask(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }
            let sa = sockaddr_union::from(req.ifr_ifru.ifru_addr);
            Ok(std::net::SocketAddr::try_from(sa)?.ip())
        }
    }

    fn set_netmask(&mut self, value: IpAddr) -> Result<()> {
        let IpAddr::V4(value) = value else {
            unimplemented!("do not support IPv6 yet")
        };
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            // Note: Here should be `ifru_netmask`, but it is not defined in `ifreq`.
            ipaddr_to_sockaddr(value, 0, &mut req.ifr_ifru.ifru_addr, OVERWRITE_SIZE);
            if let Err(err) = siocsifnetmask(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            if let Some(mut route) = self.route {
                route.netmask = value;
                self.set_route(route)?;
            }
            Ok(())
        }
    }

    fn mtu(&self) -> Result<u16> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;

            if let Err(err) = siocgifmtu(ctl.as_raw_fd(), &mut req) {
                return Err(io::Error::from(err).into());
            }

            req.ifr_ifru
                .ifru_mtu
                .try_into()
                .map_err(|_| Error::TryFromIntError)
        }
    }

    fn set_mtu(&mut self, value: u16) -> Result<()> {
        let ctl = self.ctl.as_ref().ok_or(Error::InvalidConfig)?;
        unsafe {
            let mut req = self.request()?;
            req.ifr_ifru.ifru_mtu = value as i32;

            if let Err(err) = siocsifmtu(ctl.as_raw_fd(), &req) {
                return Err(io::Error::from(err).into());
            }
            self.tun.set_mtu(value);
            Ok(())
        }
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

/// Runs a command and returns an error if the command fails, just convenience for users.
#[doc(hidden)]
pub fn run_command(command: &str, args: &[&str]) -> std::io::Result<Vec<u8>> {
    let out = std::process::Command::new(command).args(args).output()?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(if out.stderr.is_empty() {
            &out.stdout
        } else {
            &out.stderr
        });
        let info = format!("{} failed with: \"{}\"", command, err);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, info));
    }
    Ok(out.stdout)
}
