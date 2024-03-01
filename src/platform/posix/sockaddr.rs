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
use libc::{in_addr, sockaddr, sockaddr_in};
use std::{mem, net::Ipv4Addr, ptr};

/// A wrapper for `sockaddr_in`.
#[derive(Copy, Clone, Debug)]
pub struct SockAddr(sockaddr_in);

impl SockAddr {
    /// Create a new `SockAddr` from a generic `sockaddr`.
    pub fn new(value: &sockaddr) -> Result<Self> {
        if value.sa_family != libc::AF_INET as libc::sa_family_t {
            return Err(Error::InvalidAddress);
        }

        unsafe { Self::unchecked(value) }
    }

    /// # Safety
    ///  Create a new `SockAddr` and not check the source.
    pub unsafe fn unchecked(value: &sockaddr) -> Result<Self> {
        Ok(SockAddr(ptr::read(value as *const _ as *const _)))
    }

    /// # Safety
    /// Get a generic pointer to the `SockAddr`.
    pub unsafe fn as_ptr(&self) -> *const sockaddr {
        &self.0 as *const _ as *const sockaddr
    }
}

impl From<Ipv4Addr> for SockAddr {
    fn from(ip: Ipv4Addr) -> SockAddr {
        let octets = ip.octets();
        let mut addr = unsafe { mem::zeroed::<sockaddr_in>() };

        addr.sin_family = libc::AF_INET as libc::sa_family_t;
        addr.sin_port = 0;
        addr.sin_addr = in_addr {
            s_addr: u32::from_ne_bytes(octets),
        };

        SockAddr(addr)
    }
}

impl From<SockAddr> for Ipv4Addr {
    fn from(addr: SockAddr) -> Ipv4Addr {
        let ip = addr.0.sin_addr.s_addr;
        let [a, b, c, d] = ip.to_ne_bytes();

        Ipv4Addr::new(a, b, c, d)
    }
}

impl From<SockAddr> for sockaddr {
    fn from(addr: SockAddr) -> sockaddr {
        unsafe { mem::transmute(addr.0) }
    }
}

impl From<SockAddr> for sockaddr_in {
    fn from(addr: SockAddr) -> sockaddr_in {
        addr.0
    }
}

#[test]
fn test_sockaddr() {
    let old = Ipv4Addr::new(127, 0, 0, 1);
    let addr = SockAddr::from(old);
    if cfg!(target_endian = "big") {
        assert_eq!(0x7f000001, addr.0.sin_addr.s_addr);
    } else if cfg!(target_endian = "little") {
        assert_eq!(0x0100007f, addr.0.sin_addr.s_addr);
    } else {
        unreachable!();
    }
    let ip = Ipv4Addr::from(addr);
    assert_eq!(ip, old);
}
