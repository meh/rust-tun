//! Bindings to internal Linux stuff.

use libc::{c_void, c_char, c_uchar, c_short, c_ushort, c_int, c_uint, c_ulong};
use libc::{sockaddr};

pub const IFNAMSIZ: usize = 16;

pub const IFF_UP:      c_short = 0x1;
pub const IFF_RUNNING: c_short = 0x40;

pub const IFF_TUN:   c_short = 0x0001;
pub const IFF_NO_PI: c_short = 0x1000;

#[repr(C)]
pub struct ifmap {
	pub mem_start: c_ulong,
	pub mem_end:   c_ulong,
	pub base_addr: c_ushort,
	pub irq:       c_uchar,
	pub dma:       c_uchar,
	pub port:      c_uchar,
}

#[repr(C)]
pub union ifsu {
	pub raw_hdlc_proto: *mut c_void,
	pub cisco:          *mut c_void,
	pub fr:             *mut c_void,
	pub fr_pvc:         *mut c_void,
	pub fr_pvc_info:    *mut c_void,
	pub sync:           *mut c_void,
	pub te1:            *mut c_void,
}

#[repr(C)]
pub struct if_settings {
	pub type_: c_uint,
	pub size:  c_uint,
	pub ifsu:  ifsu,
}

#[repr(C)]
pub union ifrn {
	pub name: [c_char; IFNAMSIZ],
}

#[repr(C)]
pub union ifru {
	pub addr:      sockaddr,
	pub dstaddr:   sockaddr,
	pub broadaddr: sockaddr,
	pub netmask:   sockaddr,
	pub hwaddr:    sockaddr,

	pub flags:    c_short,
	pub ivalue:   c_int,
	pub mtu:      c_int,
	pub map:      ifmap,
	pub slave:    [c_char; IFNAMSIZ],
	pub newname:  [c_char; IFNAMSIZ],
	pub data:     *mut c_void,
	pub settings: if_settings,
}

#[repr(C)]
pub struct ifreq {
	pub ifrn: ifrn,
	pub ifru: ifru,
}

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
