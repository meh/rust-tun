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

use std::net::IpAddr;
#[cfg(unix)]
use std::os::unix::io::RawFd;

use crate::address::IntoAddress;
use crate::platform::PlatformConfig;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub(crate) struct WinHandle(std::os::windows::raw::HANDLE);
        unsafe impl Send for WinHandle {}
        unsafe impl Sync for WinHandle {}
    }
}

/// TUN interface OSI layer of operation.
#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub enum Layer {
    L2,
    #[default]
    L3,
}

/// Configuration builder for a TUN interface.
#[derive(Clone, Default, Debug)]
pub struct Configuration {
    pub(crate) tun_name: Option<String>,
    pub(crate) platform_config: PlatformConfig,
    pub(crate) address: Option<IpAddr>,
    pub(crate) destination: Option<IpAddr>,
    pub(crate) broadcast: Option<IpAddr>,
    pub(crate) netmask: Option<IpAddr>,
    pub(crate) mtu: Option<u16>,
    pub(crate) enabled: Option<bool>,
    pub(crate) layer: Option<Layer>,
    pub(crate) queues: Option<usize>,
    #[cfg(unix)]
    pub(crate) raw_fd: Option<RawFd>,
    #[cfg(not(unix))]
    pub(crate) raw_fd: Option<i32>,
    #[cfg(windows)]
    pub(crate) raw_handle: Option<WinHandle>,
    #[cfg(windows)]
    pub(crate) ring_capacity: Option<u32>,
    #[cfg(windows)]
    pub(crate) metric: Option<u16>,
    #[cfg(unix)]
    pub(crate) close_fd_on_drop: Option<bool>,
}

impl Configuration {
    /// Access the platform-dependent configuration.
    pub fn platform_config<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut PlatformConfig),
    {
        f(&mut self.platform_config);
        self
    }

    /// Functionally equivalent to `tun_name`
    #[deprecated(
        since = "1.1.2",
        note = "Since the API `name` may have an easy name conflict when IDE prompts, it is replaced by `tun_name` for better coding experience"
    )]
    pub fn name<S: AsRef<str>>(&mut self, tun_name: S) -> &mut Self {
        self.tun_name = Some(tun_name.as_ref().into());
        self
    }

    /// Set the tun name.
    ///
    /// [Note: on macOS, the tun name must be the form `utunx` where `x` is a number, such as `utun3`. -- end note]
    pub fn tun_name<S: AsRef<str>>(&mut self, tun_name: S) -> &mut Self {
        self.tun_name = Some(tun_name.as_ref().into());
        self
    }

    /// Set the address.
    pub fn address<A: IntoAddress>(&mut self, value: A) -> &mut Self {
        self.address = Some(value.into_address().unwrap());
        self
    }

    /// Set the destination address.
    pub fn destination<A: IntoAddress>(&mut self, value: A) -> &mut Self {
        self.destination = Some(value.into_address().unwrap());
        self
    }

    /// Set the broadcast address.
    pub fn broadcast<A: IntoAddress>(&mut self, value: A) -> &mut Self {
        self.broadcast = Some(value.into_address().unwrap());
        self
    }

    /// Set the netmask.
    pub fn netmask<A: IntoAddress>(&mut self, value: A) -> &mut Self {
        self.netmask = Some(value.into_address().unwrap());
        self
    }

    /// Set the MTU.
    ///
    /// [Note: mtu on the Windows platform is always 65535 due to wintun -- end note]
    pub fn mtu(&mut self, value: u16) -> &mut Self {
        // mtu on windows platform is always 65535 due to wintun
        if cfg!(target_family = "unix") {
            self.mtu = Some(value);
        }
        self
    }

    /// Set the interface to be enabled once created.
    pub fn up(&mut self) -> &mut Self {
        self.enabled = Some(true);
        self
    }

    /// Set the interface to be disabled once created.
    pub fn down(&mut self) -> &mut Self {
        self.enabled = Some(false);
        self
    }

    /// Set the OSI layer of operation.
    pub fn layer(&mut self, value: Layer) -> &mut Self {
        self.layer = Some(value);
        self
    }

    /// Set the number of queues.
    /// Note: The queues must be 1, otherwise will failed.
    #[deprecated(since = "1.0.0", note = "The queues will always be 1.")]
    pub fn queues(&mut self, value: usize) -> &mut Self {
        self.queues = Some(value);
        self
    }

    /// Set the raw fd.
    #[cfg(unix)]
    pub fn raw_fd(&mut self, fd: RawFd) -> &mut Self {
        self.raw_fd = Some(fd);
        self
    }
    #[cfg(not(unix))]
    pub fn raw_fd(&mut self, fd: i32) -> &mut Self {
        self.raw_fd = Some(fd);
        self
    }
    #[cfg(windows)]
    pub fn raw_handle(&mut self, handle: std::os::windows::raw::HANDLE) -> &mut Self {
        self.raw_handle = Some(WinHandle(handle));
        self
    }
    #[cfg(windows)]
    pub fn ring_capacity(&mut self, ring_capacity: u32) -> &mut Self {
        self.ring_capacity = Some(ring_capacity);
        self
    }
    #[cfg(windows)]
    pub fn metric(&mut self, metric: u16) -> &mut Self {
        self.metric = Some(metric);
        self
    }
    /// Set whether to close the received raw file descriptor on drop or not.
    /// The default behaviour is to close the received or tun2 generated file descriptor.
    /// Note: If this is set to false, it is up to the caller to ensure the
    /// file descriptor that they pass via [Configuration::raw_fd] is properly closed.
    #[cfg(unix)]
    pub fn close_fd_on_drop(&mut self, value: bool) -> &mut Self {
        self.close_fd_on_drop = Some(value);
        self
    }
}
