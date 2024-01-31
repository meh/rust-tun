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
#[cfg(windows)]
use std::os::windows::raw::HANDLE;

use crate::address::IntoAddress;
use crate::platform::PlatformConfig;

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
    pub(crate) name: Option<String>,
    pub(crate) platform_config: PlatformConfig,
    pub(crate) address: Option<IpAddr>,
    pub(crate) destination: Option<IpAddr>,
    pub(crate) broadcast: Option<IpAddr>,
    pub(crate) netmask: Option<IpAddr>,
    pub(crate) mtu: Option<usize>,
    pub(crate) enabled: Option<bool>,
    pub(crate) layer: Option<Layer>,
    pub(crate) queues: Option<usize>,
    #[cfg(unix)]
    pub(crate) raw_fd: Option<RawFd>,
    #[cfg(not(unix))]
    pub(crate) raw_fd: Option<i32>,
    #[cfg(windows)]
    pub(crate) raw_handle: Option<HANDLE>,
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

    /// Set the name.
    ///
    /// [Note: on macOS, the name must be the form `utunxxx` where `xxx` is a number. -- end note]
    pub fn name<S: AsRef<str>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.as_ref().into());
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
    pub fn mtu(&mut self, value: usize) -> &mut Self {
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
    pub fn raw_handle(&mut self, handle: HANDLE) -> &mut Self {
        self.raw_handle = Some(handle);
        self
    }
}
