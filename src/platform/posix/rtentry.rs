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

use crate::posix::sockaddr::sockaddr_union;
use crate::route::{RTF_GATEWAY, RTF_UP, RouteEntry};

impl From<&RouteEntry> for libc::rtentry {
    fn from(value: &RouteEntry) -> libc::rtentry {
        let rt_dst = value
            .rt_dst()
            .expect("Route destination address is required.");

        let rt_gateway = value
            .rt_gateway()
            .expect("Route gateway address is required.");

        let rt_genmask = value.rt_genmask().expect("Route subnet mask is required.");

        let rt_dev: *mut libc::c_char = std::ptr::null_mut();

        let rt_pad4 = {
            cfg_if::cfg_if! {
                if #[cfg(all(target_arch = "arm", target_env = "ohos"))] {
                    [value.rt_pad4().unwrap_or(0)]
                } else if #[cfg(target_pointer_width = "64")] {
                    value.rt_pad4().unwrap_or([0, 0, 0])
                } else {
                    value.rt_pad4().unwrap_or(0 as libc::c_short)
                }
            }
        };

        libc::rtentry {
            rt_pad1: value.rt_pad1().unwrap_or(0),
            rt_dst: unsafe { sockaddr_union::from((rt_dst, 0)).addr },
            rt_gateway: unsafe { sockaddr_union::from((rt_gateway, 0)).addr },
            rt_genmask: unsafe { sockaddr_union::from((rt_genmask, 0)).addr },
            rt_flags: value.rt_flags().unwrap_or(RTF_GATEWAY | RTF_UP),
            rt_pad2: value.rt_pad2().unwrap_or(0),
            rt_pad3: value.rt_pad3().unwrap_or(0),
            rt_tos: value.rt_tos().unwrap_or(0),
            rt_class: value.rt_class().unwrap_or(0),
            rt_pad4,
            rt_metric: value.rt_metric().unwrap_or(0),
            rt_dev: value.rt_dev().unwrap_or(rt_dev),
            rt_mtu: value.rt_mtu().unwrap_or(1500),
            rt_window: value.rt_window().unwrap_or(0),
            rt_irtt: value.rt_irtt().unwrap_or(0),
        }
    }
}
