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

//! Bindings to internal Linux stuff.

use ioctl::*;
use libc::{c_int, ifreq};

ioctl!(bad read siocgifflags with 0x8913; ifreq);
ioctl!(bad write siocsifflags with 0x8914; ifreq);
ioctl!(bad read siocgifaddr with 0x8915; ifreq);
ioctl!(bad write siocsifaddr with 0x8916; ifreq);
ioctl!(bad read siocgifdstaddr with 0x8917; ifreq);
ioctl!(bad write siocsifdstaddr with 0x8918; ifreq);
ioctl!(bad read siocgifbrdaddr with 0x8919; ifreq);
ioctl!(bad write siocsifbrdaddr with 0x891a; ifreq);
ioctl!(bad read siocgifnetmask with 0x891b; ifreq);
ioctl!(bad write siocsifnetmask with 0x891c; ifreq);
ioctl!(bad read siocgifmtu with 0x8921; ifreq);
ioctl!(bad write siocsifmtu with 0x8922; ifreq);
ioctl!(bad write siocsifname with 0x8923; ifreq);

ioctl!(write tunsetiff with b'T', 202; c_int);
ioctl!(write tunsetpersist with b'T', 203; c_int);
ioctl!(write tunsetowner with b'T', 204; c_int);
ioctl!(write tunsetgroup with b'T', 206; c_int);
