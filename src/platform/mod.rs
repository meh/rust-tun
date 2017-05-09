#[cfg(unix)]
mod posix;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use self::linux::{Device, create, next};
