use std::mem;
use std::net::{Ipv4Addr};

use libc::{c_ushort, c_int, c_uint};
use libc::{sockaddr, sockaddr_in, in_addr};
use libc::{AF_INET};

use error::*;

pub unsafe fn to_sockaddr(ip: Ipv4Addr) -> Result<sockaddr> {
	let     ip   = ip.octets();
	let mut addr = mem::zeroed::<sockaddr_in>();

	addr.sin_family = AF_INET as c_ushort;
	addr.sin_port   = 0;
	addr.sin_addr   = in_addr { s_addr:
		((ip[3] as c_uint) << 24) |
		((ip[2] as c_uint) << 16) |
		((ip[1] as c_uint) <<  8) |
		((ip[0] as c_uint))
	};

	Ok(mem::transmute(addr))
}

pub unsafe fn from_sockaddr(value: &sockaddr) -> Result<Ipv4Addr> {
	match value.sa_family as c_int {
		AF_INET => {
			let addr = mem::transmute::<_, &sockaddr_in>(value);
			let ip   = addr.sin_addr.s_addr;

			Ok(Ipv4Addr::new(
				((ip      ) & 0xff) as u8,
				((ip >>  8) & 0xff) as u8,
				((ip >> 16) & 0xff) as u8,
				((ip >> 24) & 0xff) as u8))
		}

		_ =>
			Err(ErrorKind::UnsupportedFamily.into())
	}
}
