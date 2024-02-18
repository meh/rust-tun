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

//! Bindings to internal macOS stuff.

use libc::{c_char, c_int, c_short, c_uint, c_ushort, c_void, sockaddr, IFNAMSIZ};
use nix::{ioctl_readwrite, ioctl_write_ptr};

pub const UTUN_CONTROL_NAME: &str = "com.apple.net.utun_control";

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ctl_info {
    pub ctl_id: c_uint,
    pub ctl_name: [c_char; 96],
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_ctl {
    pub sc_len: c_char,
    pub sc_family: c_char,
    pub ss_sysaddr: c_ushort,
    pub sc_id: c_uint,
    pub sc_unit: c_uint,
    pub sc_reserved: [c_uint; 5],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ifrn {
    pub name: [c_char; IFNAMSIZ],
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifdevmtu {
    pub current: c_int,
    pub min: c_int,
    pub max: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ifku {
    pub ptr: *mut c_void,
    pub value: c_int,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifkpi {
    pub module_id: c_uint,
    pub type_: c_uint,
    pub ifku: ifku,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union ifru {
    pub addr: sockaddr,
    pub dstaddr: sockaddr,
    pub broadaddr: sockaddr,

    pub flags: c_short,
    pub metric: c_int,
    pub mtu: c_int,
    pub phys: c_int,
    pub media: c_int,
    pub intval: c_int,
    pub data: *mut c_void,
    pub devmtu: ifdevmtu,
    pub wake_flags: c_uint,
    pub route_refcnt: c_uint,
    pub cap: [c_int; 2],
    pub functional_type: c_uint,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifreq {
    pub ifrn: ifrn,
    pub ifru: ifru,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifaliasreq {
    pub ifran: [c_char; IFNAMSIZ],
    pub addr: sockaddr,
    pub broadaddr: sockaddr,
    pub mask: sockaddr,
}

ioctl_readwrite!(ctliocginfo, b'N', 3, ctl_info);

ioctl_write_ptr!(siocsifflags, b'i', 16, ifreq);
ioctl_readwrite!(siocgifflags, b'i', 17, ifreq);

ioctl_write_ptr!(siocsifaddr, b'i', 12, ifreq);
ioctl_readwrite!(siocgifaddr, b'i', 33, ifreq);

ioctl_write_ptr!(siocsifdstaddr, b'i', 14, ifreq);
ioctl_readwrite!(siocgifdstaddr, b'i', 34, ifreq);

ioctl_write_ptr!(siocsifbrdaddr, b'i', 19, ifreq);
ioctl_readwrite!(siocgifbrdaddr, b'i', 35, ifreq);

ioctl_write_ptr!(siocsifnetmask, b'i', 22, ifreq);
ioctl_readwrite!(siocgifnetmask, b'i', 37, ifreq);

ioctl_write_ptr!(siocsifmtu, b'i', 52, ifreq);
ioctl_readwrite!(siocgifmtu, b'i', 51, ifreq);

ioctl_write_ptr!(siocaifaddr, b'i', 26, ifaliasreq);
ioctl_write_ptr!(siocdifaddr, b'i', 25, ifreq);
