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

use std::net::{IpAddr, Ipv4Addr};
use std::net::{SocketAddr, SocketAddrV4};

use error::Error;

/// Helper trait to convert things into IPv4 addresses.
pub trait IntoAddress {
	/// Convert the type to an `Ipv4Addr`.
	fn into_address(&self) -> Result<Ipv4Addr, Error>;
}

impl IntoAddress for u32 {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		Ok(Ipv4Addr::new(
			((*self      ) & 0xff) as u8,
			((*self >>  8) & 0xff) as u8,
			((*self >> 16) & 0xff) as u8,
			((*self >> 24) & 0xff) as u8))
	}
}

impl IntoAddress for i32 {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(*self as u32).into_address()
	}
}

impl IntoAddress for (u8, u8, u8, u8) {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		Ok(Ipv4Addr::new(self.0, self.1, self.2, self.3))
	}
}

impl IntoAddress for str {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		self.parse().map_err(|_| Error::InvalidAddress)
	}
}

impl<'a> IntoAddress for &'a str {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(*self).into_address()
	}
}

impl IntoAddress for String {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(&**self).into_address()
	}
}

impl<'a> IntoAddress for &'a String {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(&**self).into_address()
	}
}

impl IntoAddress for Ipv4Addr {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		Ok(self.clone())
	}
}

impl<'a> IntoAddress for &'a Ipv4Addr {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(&**self).into_address()
	}
}

impl IntoAddress for IpAddr {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		match *self {
			IpAddr::V4(ref value) =>
				Ok(value.clone()),

			IpAddr::V6(_) =>
				Err(Error::InvalidAddress)
		}
	}
}

impl<'a> IntoAddress for &'a IpAddr {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(&**self).into_address()
	}
}

impl IntoAddress for SocketAddrV4 {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		Ok(self.ip().clone())
	}
}

impl<'a> IntoAddress for &'a SocketAddrV4 {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(&**self).into_address()
	}
}

impl IntoAddress for SocketAddr {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		match *self {
			SocketAddr::V4(ref value) =>
				Ok(value.ip().clone()),

			SocketAddr::V6(_) =>
				Err(Error::InvalidAddress)
		}
	}
}

impl<'a> IntoAddress for &'a SocketAddr {
	fn into_address(&self) -> Result<Ipv4Addr, Error> {
		(&**self).into_address()
	}
}
