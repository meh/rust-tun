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
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
pub(crate) use sockaddr::sockaddr_union;

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) use sockaddr::ipaddr_to_sockaddr;

mod fd;
pub(crate) use self::fd::Fd;

mod split;
pub use self::split::{Reader, Tun, Writer};

#[allow(dead_code)]
pub fn tun_name_to_index(name: impl AsRef<str>) -> std::io::Result<u32> {
    let name_cstr = std::ffi::CString::new(name.as_ref()).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid interface name")
    })?;
    let result = unsafe { libc::if_nametoindex(name_cstr.as_ptr()) };
    if result == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(result as _)
    }
}
