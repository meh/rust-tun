//! POSIX compliant support.

mod sockaddr;
pub use self::sockaddr::SockAddr;

mod fd;
pub use self::fd::Fd;

mod split;
pub use self::split::{Reader, Writer};
