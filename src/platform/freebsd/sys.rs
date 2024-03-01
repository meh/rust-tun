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

//! Bindings to internal FreeBSD stuff.

use libc::{c_char, c_int, c_uint, ifreq, sockaddr, IFNAMSIZ};
use nix::{ioctl_readwrite, ioctl_write_ptr};

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
pub struct ifaliasreq {
    pub ifran: [c_char; IFNAMSIZ],
    pub addr: sockaddr,
    pub dstaddr: sockaddr,
    pub mask: sockaddr,
    pub ifra_vhid: c_int,
}

// #[allow(non_camel_case_types)]
// #[repr(C)]
// #[derive(Copy, Clone)]
// pub struct in_aliasreq  {
//     pub ifra_name: [c_char; IFNAMSIZ],
//     pub ifra_addr: sockaddr_in,
//     pub ifra_dstaddr: sockaddr_in,
//     pub ifra_mask: sockaddr_in,
// 	pub ifra_vhid:c_int
// }

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

ioctl_write_ptr!(siocaifaddr, b'i', 43, ifaliasreq);
ioctl_write_ptr!(siocdifaddr, b'i', 25, ifreq);

ioctl_write_ptr!(siocifcreate, b'i', 122, ifreq);

ioctl_write_ptr!(siocsifphyaddr, b'i', 70, ifaliasreq);

ioctl_write_ptr!(siocsifname, b'i', 40, ifreq);
