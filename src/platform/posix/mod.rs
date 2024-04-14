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

//! POSIX compliant support.

mod sockaddr;
#[cfg(any(target_os = "freebsd", target_os = "macos"))]
pub(crate) use sockaddr::rs_addr_to_sockaddr;
pub(crate) use sockaddr::{ipaddr_to_sockaddr, sockaddr_to_rs_addr, sockaddr_union};

mod fd;
pub(crate) use self::fd::Fd;

mod split;
pub use self::split::{Reader, Tun, Writer};
