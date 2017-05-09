#![feature(untagged_unions)]
#![recursion_limit = "1024"]

extern crate libc;
#[macro_use]
extern crate error_chain;

#[cfg(target_os = "linux")]
#[macro_use]
extern crate ioctl_sys as ioctl;

mod error;
pub use error::*;

mod device;
pub use device::Device;

pub mod platform;
pub use platform::{create, next};
