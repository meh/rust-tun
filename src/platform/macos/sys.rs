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

use ioctl::*;
use libc::{c_char, c_uint, ifreq, sockaddr, IFNAMSIZ};

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
pub struct ifaliasreq {
    pub ifran: [c_char; IFNAMSIZ],
    pub addr: sockaddr,
    pub broadaddr: sockaddr,
    pub mask: sockaddr,
}

ioctl!(readwrite ctliocginfo with 'N', 3; ctl_info);

ioctl!(write siocsifflags with 'i', 16; ifreq);
ioctl!(readwrite siocgifflags with 'i', 17; ifreq);

ioctl!(write siocsifaddr with 'i', 12; ifreq);
ioctl!(readwrite siocgifaddr with 'i', 33; ifreq);

ioctl!(write siocsifdstaddr with 'i', 14; ifreq);
ioctl!(readwrite siocgifdstaddr with 'i', 34; ifreq);

ioctl!(write siocsifbrdaddr with 'i', 19; ifreq);
ioctl!(readwrite siocgifbrdaddr with 'i', 35; ifreq);

ioctl!(write siocsifnetmask with 'i', 22; ifreq);
ioctl!(readwrite siocgifnetmask with 'i', 37; ifreq);

ioctl!(write siocsifmtu with 'i', 52; ifreq);
ioctl!(readwrite siocgifmtu with 'i', 51; ifreq);

ioctl!(write siocaifaddr with 'i', 26; ifaliasreq);
ioctl!(write siocdifaddr with 'i', 25; ifreq);
