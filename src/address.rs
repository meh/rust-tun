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

use crate::error::{Error, Result};
use std::net::{IpAddr, Ipv4Addr};
use std::net::{SocketAddr, SocketAddrV4};

/// Helper trait to convert things into IPv4 addresses.
pub trait ToAddress {
    /// Convert the type to an `Ipv4Addr`.
    fn to_address(&self) -> Result<IpAddr>;
}

impl ToAddress for u32 {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(IpAddr::V4(Ipv4Addr::new(
            ((*self) & 0xff) as u8,
            ((*self >> 8) & 0xff) as u8,
            ((*self >> 16) & 0xff) as u8,
            ((*self >> 24) & 0xff) as u8,
        )))
    }
}

impl ToAddress for i32 {
    fn to_address(&self) -> Result<IpAddr> {
        (*self as u32).to_address()
    }
}

impl ToAddress for (u8, u8, u8, u8) {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(IpAddr::V4(Ipv4Addr::new(self.0, self.1, self.2, self.3)))
    }
}

impl ToAddress for str {
    fn to_address(&self) -> Result<IpAddr> {
        self.parse().map_err(|_| Error::InvalidAddress)
    }
}

impl ToAddress for &str {
    fn to_address(&self) -> Result<IpAddr> {
        (*self).to_address()
    }
}

impl ToAddress for String {
    fn to_address(&self) -> Result<IpAddr> {
        self.as_str().to_address()
    }
}

impl ToAddress for &String {
    fn to_address(&self) -> Result<IpAddr> {
        self.as_str().to_address()
    }
}

impl ToAddress for Ipv4Addr {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(IpAddr::V4(*self))
    }
}

impl ToAddress for &Ipv4Addr {
    fn to_address(&self) -> Result<IpAddr> {
        (*self).to_address()
    }
}

impl ToAddress for IpAddr {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(*self)
    }
}

impl ToAddress for &IpAddr {
    fn to_address(&self) -> Result<IpAddr> {
        (*self).to_address()
    }
}

impl ToAddress for SocketAddrV4 {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(IpAddr::V4(*self.ip()))
    }
}

impl ToAddress for &SocketAddrV4 {
    fn to_address(&self) -> Result<IpAddr> {
        (*self).to_address()
    }
}

impl ToAddress for SocketAddr {
    fn to_address(&self) -> Result<IpAddr> {
        Ok(self.ip())
    }
}

impl ToAddress for &SocketAddr {
    fn to_address(&self) -> Result<IpAddr> {
        (*self).to_address()
    }
}
