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

use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::sync::Arc;
use std::str::FromStr;
use std::net::Ipv4Addr;
use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;

use libc;
use libc::{SOCK_DGRAM, AF_INET, socklen_t, sockaddr, c_void, c_char};

use error::*;
use device::Device as D;
use platform::macos::sys::*;
use configuration::Configuration;
use platform::posix::{self, SockAddr, Fd};


/// A TUN device using the TUN macOS driver.
pub struct Device {
	name: String,
	tun: Fd,
	ctl: Fd,
}

impl Device {
	/// Create a new `Device` for the given `Configuration`.
	pub fn new(config: &Configuration) -> Result<Self> {
		let dev_id = match config.name.as_ref() {
			Some(name) => {
				if name.len() > IFNAMSIZ {
					return Err(ErrorKind::NameTooLong.into());
				}
				if !name.starts_with("utun") {
					return Err(ErrorKind::InvalidName.into());
				}
				u8::from_str(&name[4..])? as u8
			}

			None => 0,
		};

		let mut device;
		unsafe {
			let tun = libc::socket(PF_SYSTEM, SOCK_DGRAM, SYSPROTO_CONTROL);
			if tun < 0 {
				return Err(io::Error::last_os_error().into());
			}

			let mut info = ctl_info {
				ctl_id: 0,
				ctl_name: {
					let mut buffer = [0u8; 96];
					buffer[..UTUN_CONTROL_NAME.len()].clone_from_slice(UTUN_CONTROL_NAME.as_bytes());
					buffer
				},
			};

			if ctliocginfo(tun, &mut info as *mut _ as *mut _) < 0 {
				libc::close(tun);
				return Err(io::Error::last_os_error().into());
			}

			let addr = sockaddr_ctl {
				sc_id: info.ctl_id,
				sc_len: mem::size_of::<sockaddr_ctl>() as u8,
				sc_family: AF_SYSTEM,
				ss_sysaddr: AF_SYS_CONTROL,
				sc_unit: dev_id as u32 + 1,
				sc_reserved: [0; 5],
			};

			if libc::connect(tun, &addr as *const sockaddr_ctl as *const sockaddr, mem::size_of_val(&addr) as socklen_t) < 0 {
				return Err(io::Error::last_os_error().into());
			}

			let mut name_buf = [0u8; 64];
			let mut name_length: socklen_t = 64;
			if libc::getsockopt(
				tun, 
				SYSPROTO_CONTROL, 
				UTUN_OPT_IFNAME, 
				&mut name_buf as *mut _ as *mut c_void, 
				&mut name_length as *mut socklen_t) < 0 {
				return Err(io::Error::last_os_error().into());
			}

			let ctl = libc::socket(AF_INET, SOCK_DGRAM, 0);
			if ctl < 0 {
				return Err(io::Error::last_os_error().into());
			}

			device = Device {
				name: CStr::from_ptr(name_buf.as_ptr() as *const c_char).to_string_lossy().into(),
				tun: Fd(tun),
				ctl: Fd(ctl),
			};
		}

		device.configure(&config)?;
		Ok(device)
	}

	/// Prepare a new request.
	pub unsafe fn request(&self) -> ifreq {
		let mut req: ifreq = mem::zeroed();
		ptr::copy_nonoverlapping(self.name.as_ptr() as *const c_char, req.ifrn.name.as_mut_ptr(), self.name.len());

		req
	}

	/// Prepare a new alias request.
	pub unsafe fn alias_request(&self) -> ifaliasreq {
		let mut alias_req: ifaliasreq = mem::zeroed();
		ptr::copy_nonoverlapping(self.name.as_ptr() as *const c_char, alias_req.ifran.as_mut_ptr(), self.name.len());

		alias_req
	}

	/// Set the ipv4 alias of the device.
	pub fn ipv4(&mut self, addr: Ipv4Addr, broadaddr: Ipv4Addr, mask: Ipv4Addr) -> Result<()> {
		unsafe {
			let mut alias_req = self.alias_request();
			alias_req.addr = SockAddr::from(addr).into();
			alias_req.broadaddr = SockAddr::from(broadaddr).into();
			alias_req.mask = SockAddr::from(mask).into();

			if siocaifaddr(self.ctl.as_raw_fd(), &alias_req) < 0 {
				return Err(io::Error::last_os_error().into());
			}
			Ok(())
		}
	}

	pub fn delete_addr(&mut self) -> Result<()> {
		unsafe {
			let req = self.request();

			if siocdifaddr(self.ctl.as_raw_fd(), &req) < 0 {
				return Err(io::Error::last_os_error().into());
			}
			Ok(())
		}
	}

	/// Split the interface into a `Reader` and `Writer`.
	pub fn split(self) -> (posix::Reader, posix::Writer) {
		let fd = Arc::new(self.tun);
		(posix::Reader(fd.clone()), posix::Writer(fd.clone()))
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

impl D for Device {
	fn name(&self) -> &str {
		&self.name
	}

	/// Cannot set interface name on Darwin
	fn set_name(&mut self, value: &str) -> Result<()> {
		unimplemented!()
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
		unsafe {
			let mut req = self.request();
			req.ifru.addr = SockAddr::from(value).into();

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

			SockAddr::new(&req.ifru.dstaddr).map(Into::into)
		}
	}

	fn set_destination(&mut self, value: Ipv4Addr) -> Result<()> {
		unsafe {
			let mut req = self.request();
			req.ifru.dstaddr = SockAddr::from(value).into();

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

			SockAddr::new(&req.ifru.broadaddr).map(Into::into)
		}
	}

	fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<()> {
		unsafe {
			let mut req = self.request();
			req.ifru.broadaddr = SockAddr::from(value).into();

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

			SockAddr::unchecked(&req.ifru.addr).map(Into::into)
		}
	}

	fn set_netmask(&mut self, value: Ipv4Addr) -> Result<()> {
		unsafe {
			let mut req = self.request();
			req.ifru.addr = SockAddr::from(value).into();

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

			Ok(req.ifru.mtu)
		}
	}

	fn set_mtu(&mut self, value: i32) -> Result<()> {
		unsafe {
			let mut req = self.request();
			req.ifru.mtu = value;

			if siocsifmtu(self.ctl.as_raw_fd(), &req) < 0 {
				return Err(io::Error::last_os_error().into());
			}

			Ok(())
		}
	}
}

#[cfg(feature = "mio")]
mod mio {
	use std::io;
	use mio::{Ready, Poll, PollOpt, Token};
	use mio::event::Evented;

	impl Evented for super::Device {
		fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
			self.tun.register(poll, token, interest, opts)
		}

		fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
			self.tun.reregister(poll, token, interest, opts)
		}

		fn deregister(&self, poll: &Poll) -> io::Result<()> {
			self.tun.deregister(poll)
		}
	}
}
