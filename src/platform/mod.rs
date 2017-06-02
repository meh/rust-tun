//! Platform specific modules.

#[cfg(unix)]
pub mod posix;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{Device, Configuration, create};
