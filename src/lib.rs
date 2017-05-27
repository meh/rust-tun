#![feature(untagged_unions)]
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
pub use platform::{configure, create, next};
