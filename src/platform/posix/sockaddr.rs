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

/// # Safety
unsafe fn sockaddr_to_rs_addr(sa: &sockaddr_union) -> Option<std::net::SocketAddr> {
    match unsafe { sa.addr_stor }.ss_family as libc::c_int {
        libc::AF_INET => {
            let sa_in = unsafe { sa.addr4 };
            let ip = std::net::Ipv4Addr::from(sa_in.sin_addr.s_addr.to_ne_bytes());
            let port = u16::from_be(sa_in.sin_port);
            Some(std::net::SocketAddr::new(ip.into(), port))
        }
        libc::AF_INET6 => {
            let sa_in6 = unsafe { sa.addr6 };
            let ip = std::net::Ipv6Addr::from(sa_in6.sin6_addr.s6_addr);
            let port = u16::from_be(sa_in6.sin6_port);
            Some(std::net::SocketAddr::new(ip.into(), port))
        }
        _ => None,
    }
}

fn rs_addr_to_sockaddr(addr: std::net::SocketAddr) -> sockaddr_union {
    match addr {
        std::net::SocketAddr::V4(ipv4) => {
            let mut addr: sockaddr_union = unsafe { std::mem::zeroed() };
            #[cfg(any(target_os = "freebsd", target_os = "macos"))]
            {
                addr.addr4.sin_len = std::mem::size_of::<libc::sockaddr_in>() as u8;
            }
            addr.addr4.sin_family = libc::AF_INET as libc::sa_family_t;
            addr.addr4.sin_addr.s_addr = u32::from_ne_bytes(ipv4.ip().octets());
            addr.addr4.sin_port = ipv4.port().to_be();
            addr
        }
        std::net::SocketAddr::V6(ipv6) => {
            let mut addr: sockaddr_union = unsafe { std::mem::zeroed() };
            #[cfg(any(target_os = "freebsd", target_os = "macos"))]
            {
                addr.addr6.sin6_len = std::mem::size_of::<libc::sockaddr_in6>() as u8;
            }
            addr.addr6.sin6_family = libc::AF_INET6 as libc::sa_family_t;
            addr.addr6.sin6_addr.s6_addr = ipv6.ip().octets();
            addr.addr6.sin6_port = ipv6.port().to_be();
            addr
        }
    }
}

/// # Safety
/// Fill the `addr` with the `src_addr` and `src_port`, the `size` should be the size of overwriting
#[allow(dead_code)]
pub(crate) unsafe fn ipaddr_to_sockaddr<T>(
    src_addr: T,
    src_port: u16,
    addr: &mut libc::sockaddr,
    size: usize,
) where
    T: Into<std::net::IpAddr>,
{
    let sa = rs_addr_to_sockaddr((src_addr.into(), src_port).into());
    unsafe {
        std::ptr::copy_nonoverlapping(
            &sa as *const _ as *const libc::c_void,
            addr as *mut _ as *mut libc::c_void,
            size.min(std::mem::size_of::<sockaddr_union>()),
        )
    };
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union sockaddr_union {
    pub addr_stor: libc::sockaddr_storage,
    pub addr6: libc::sockaddr_in6,
    pub addr4: libc::sockaddr_in,
    pub addr: libc::sockaddr,
}

impl From<libc::sockaddr_storage> for sockaddr_union {
    fn from(addr: libc::sockaddr_storage) -> Self {
        sockaddr_union { addr_stor: addr }
    }
}

impl From<libc::sockaddr_in6> for sockaddr_union {
    fn from(addr: libc::sockaddr_in6) -> Self {
        sockaddr_union { addr6: addr }
    }
}

impl From<libc::sockaddr_in> for sockaddr_union {
    fn from(addr: libc::sockaddr_in) -> Self {
        sockaddr_union { addr4: addr }
    }
}

impl From<libc::sockaddr> for sockaddr_union {
    fn from(addr: libc::sockaddr) -> Self {
        sockaddr_union { addr }
    }
}

impl From<std::net::SocketAddr> for sockaddr_union {
    fn from(addr: std::net::SocketAddr) -> Self {
        rs_addr_to_sockaddr(addr)
    }
}

impl TryFrom<sockaddr_union> for std::net::SocketAddr {
    type Error = std::io::Error;

    fn try_from(addr: sockaddr_union) -> Result<Self, Self::Error> {
        unsafe { sockaddr_to_rs_addr(&addr).ok_or(std::io::ErrorKind::InvalidInput.into()) }
    }
}

impl<T: Into<std::net::IpAddr>> From<(T, u16)> for sockaddr_union {
    fn from((ip, port): (T, u16)) -> Self {
        let ip: std::net::IpAddr = ip.into();
        rs_addr_to_sockaddr(std::net::SocketAddr::new(ip, port))
    }
}

#[test]
fn test_conversion() {
    let old = std::net::SocketAddr::new([127, 0, 0, 1].into(), 0x0208);
    let addr = rs_addr_to_sockaddr(old);
    unsafe {
        if cfg!(target_endian = "big") {
            assert_eq!(0x7f000001, addr.addr4.sin_addr.s_addr);
            assert_eq!(0x0208, addr.addr4.sin_port);
        } else if cfg!(target_endian = "little") {
            assert_eq!(0x0100007f, addr.addr4.sin_addr.s_addr);
            assert_eq!(0x0802, addr.addr4.sin_port);
        } else {
            unreachable!();
        }
    };
    let ip = unsafe { sockaddr_to_rs_addr(&addr).unwrap() };
    assert_eq!(ip, old);

    let old = std::net::SocketAddr::new(std::net::Ipv6Addr::LOCALHOST.into(), 0x0208);
    let addr = rs_addr_to_sockaddr(old);
    let ip = unsafe { sockaddr_to_rs_addr(&addr).unwrap() };
    assert_eq!(ip, old);

    let old = std::net::IpAddr::V4([10, 0, 0, 33].into());
    let mut addr: sockaddr_union = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::sockaddr_in>();
    unsafe { ipaddr_to_sockaddr(old, 0x0208, &mut addr.addr, size) };
    let ip = unsafe { sockaddr_to_rs_addr(&addr).unwrap() };
    assert_eq!(ip, std::net::SocketAddr::new(old, 0x0208));
}
