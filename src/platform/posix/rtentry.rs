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

use libc::{rtentry};
use crate::platform::posix::SockAddr;
use crate::route::{RouteEntry, RTF_GATEWAY, RTF_UP};

impl From<&RouteEntry> for rtentry {
    fn from(value: &RouteEntry) -> rtentry {
        let rt_dst = value.rt_dst()
            .expect("Route destination address is required.");

        let rt_gateway = value.rt_gateway()
            .expect("Route gateway address is required.");

        let rt_genmask = value.rt_genmask()
            .expect("Route subnet mask is required.");

        let rt_dev: *mut i8 = std::ptr::null_mut();

        rtentry {
            rt_pad1: value.rt_pad1().or(Some(0)).unwrap(),
            rt_dst: SockAddr::from(rt_dst).into(),
            rt_gateway: SockAddr::from(rt_gateway).into(),
            rt_genmask: SockAddr::from(rt_genmask).into(),
            rt_flags: value.rt_flags().or(Some(RTF_GATEWAY | RTF_UP)).unwrap(),
            rt_pad2: value.rt_pad2().or(Some(0)).unwrap(),
            rt_pad3: value.rt_pad3().or(Some(0)).unwrap(),
            rt_tos: value.rt_tos().or(Some(0)).unwrap(),
            rt_class: value.rt_class().or(Some(0)).unwrap(),
            rt_pad4: value.rt_pad4().or(Some([0, 0, 0])).unwrap(),
            rt_metric: value.rt_metric().or(Some(0)).unwrap(),
            rt_dev: value.rt_dev().or(Some(rt_dev)).unwrap(),
            rt_mtu: value.rt_mtu().or(Some(1500)).unwrap(),
            rt_window: value.rt_window().or(Some(0)).unwrap(),
            rt_irtt: value.rt_irtt().or(Some(0)).unwrap()
        }
    }
}