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

use crate::ToAddress;
use std::net::IpAddr;

pub const RTF_UP: u16 = 0x0001;
pub const RTF_GATEWAY: u16 = 0x0002;
pub const RTF_HOST: u16 = 0x0004;
pub const RTF_REINSTATE: u16 = 0x0008;
pub const RTF_DYNAMIC: u16 = 0x0010;
pub const RTF_MODIFIED: u16 = 0x0020;
pub const RTF_MTU: u16 = 0x0040;
pub const RTF_WINDOW: u16 = 0x0080;
pub const RTF_IRTT: u16 = 0x0100;
pub const RTF_REJECT: u16 = 0x0200;

#[derive(Clone, Copy, Debug, Default)]
pub struct RouteEntry {
    rt_pad1: Option<u64>,
    rt_dst: Option<IpAddr>,
    rt_gateway: Option<IpAddr>,
    rt_genmask: Option<IpAddr>,
    rt_flags: Option<u16>,
    rt_pad2: Option<i16>,
    rt_pad3: Option<u64>,
    rt_tos: Option<u8>,
    rt_class: Option<u8>,
    rt_pad4: Option<[i16; 3]>,
    rt_metric: Option<i16>,
    rt_dev: Option<*mut i8>,
    rt_mtu: Option<u64>,
    rt_window: Option<u64>,
    rt_irtt: Option<u16>,
}

impl RouteEntry {
    pub fn new() -> RouteEntry {
        RouteEntry::default()
    }

    pub fn set_rt_pad1(mut self, value: u64) -> RouteEntry {
        self.rt_pad1 = Some(value);
        self
    }

    pub fn rt_pad1(&self) -> Option<u64> {
        self.rt_pad1
    }

    pub fn set_rt_dst<A: ToAddress>(mut self, value: A) -> RouteEntry {
        self.rt_dst = Some(value.to_address().unwrap());
        self
    }

    pub fn rt_dst(&self) -> Option<IpAddr> {
        self.rt_dst
    }

    pub fn set_rt_gateway<A: ToAddress>(mut self, value: A) -> RouteEntry {
        self.rt_gateway = Some(value.to_address().unwrap());
        self
    }

    pub fn rt_gateway(&self) -> Option<IpAddr> {
        self.rt_gateway
    }

    pub fn set_rt_genmask<A: ToAddress>(mut self, value: A) -> RouteEntry {
        self.rt_genmask = Some(value.to_address().unwrap());
        self
    }

    pub fn rt_genmask(&self) -> Option<IpAddr> {
        self.rt_genmask
    }

    pub fn set_rt_flags(mut self, value: u16) -> RouteEntry {
        self.rt_flags = Some(value);
        self
    }

    pub fn rt_flags(&self) -> Option<u16> {
        self.rt_flags
    }

    pub fn set_rt_pad2(mut self, value: i16) -> RouteEntry {
        self.rt_pad2 = Some(value);
        self
    }

    pub fn rt_pad2(&self) -> Option<i16> {
        self.rt_pad2
    }

    pub fn set_rt_pad3(mut self, value: u64) -> RouteEntry {
        self.rt_pad3 = Some(value);
        self
    }

    pub fn rt_pad3(&self) -> Option<u64> {
        self.rt_pad3
    }

    pub fn set_rt_tos(mut self, value: u8) -> RouteEntry {
        self.rt_tos = Some(value);
        self
    }

    pub fn rt_tos(&self) -> Option<u8> {
        self.rt_tos
    }

    pub fn set_rt_class(mut self, value: u8) -> RouteEntry {
        self.rt_class = Some(value);
        self
    }

    pub fn rt_class(&self) -> Option<u8> {
        self.rt_class
    }

    pub fn set_rt_pad4(mut self, value: [i16; 3]) -> RouteEntry {
        self.rt_pad4 = Some(value);
        self
    }

    pub fn rt_pad4(&self) -> Option<[i16; 3]> {
        self.rt_pad4
    }

    pub fn set_rt_metric(mut self, value: i16) -> RouteEntry {
        self.rt_metric = Some(value);
        self
    }

    pub fn rt_metric(&self) -> Option<i16> {
        self.rt_metric
    }

    pub fn set_rt_dev(mut self, value: *mut i8) -> RouteEntry {
        self.rt_dev = Some(value);
        self
    }

    pub fn rt_dev(&self) -> Option<*mut i8> {
        self.rt_dev
    }

    pub fn set_rt_mtu(mut self, value: u64) -> RouteEntry {
        self.rt_mtu = Some(value);
        self
    }

    pub fn rt_mtu(&self) -> Option<u64> {
        self.rt_mtu
    }

    pub fn set_rt_window(mut self, value: u64) -> RouteEntry {
        self.rt_window = Some(value);
        self
    }

    pub fn rt_window(&self) -> Option<u64> {
        self.rt_window
    }

    pub fn set_rt_irtt(mut self, value: u16) -> RouteEntry {
        self.rt_irtt = Some(value);
        self
    }

    pub fn rt_irtt(&self) -> Option<u16> {
        self.rt_irtt
    }
}
