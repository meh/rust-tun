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

//! Bindings to internal OpenBSD stuff.

use ioctl::*;
use libc::sockaddr;
use libc::{c_char, c_int, c_short, c_uint};

pub const IFNAMSIZ: usize = 16;

pub const IFF_UP: c_short = 0x1;
pub const IFF_RUNNING: c_short = 0x40;

#[repr(C)]
#[derive(Copy, Clone)]
pub union ifru {
    pub addr: sockaddr,
    pub dstaddr: sockaddr,
    pub broadaddr: sockaddr,

    pub flags: c_short,
    pub metric: c_int,
    pub vnetid: i64,
    pub media: u64,
    pub index: c_uint,
    // caddr_t ifru_data is missing
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifreq {
    pub name: [c_char; IFNAMSIZ],
    pub ifru: ifru,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifaliasreq {
    pub name: [c_char; IFNAMSIZ],
    pub addr: sockaddr,
    pub dstaddr: sockaddr,
    pub mask: sockaddr,
}

ioctl!(write siocsifflags with 'i', 16; ifreq);
ioctl!(readwrite siocgifflags with 'i', 17; ifreq);

ioctl!(readwrite siocgifaddr with 'i', 33; ifreq);

ioctl!(readwrite siocgifdstaddr with 'i', 34; ifreq);

ioctl!(readwrite siocgifnetmask with 'i', 37; ifreq);

ioctl!(write siocsifmtu with 'i', 127; ifreq);
ioctl!(readwrite siocgifmtu with 'i', 126; ifreq);

ioctl!(write siocaifaddr with 'i', 26; ifaliasreq);
