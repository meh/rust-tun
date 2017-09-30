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

#![recursion_limit = "1024"]

extern crate libc;
#[macro_use]
extern crate error_chain;

#[cfg(target_os = "linux")]
#[macro_use]
extern crate ioctl_sys as ioctl;

#[cfg(all(feature = "mio", target_os = "linux"))]
extern crate mio;

mod error;
pub use error::*;

mod address;
pub use address::IntoAddress;

mod device;
pub use device::Device;

mod configuration;
pub use configuration::Configuration;

pub mod platform;
pub use platform::create;
