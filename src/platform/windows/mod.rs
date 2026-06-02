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

//! Windows specific functionality.

mod device;
mod sys;

use crate::error::Result;
use crate::{AbstractDevice, configuration::Configuration};
pub use device::{Device, Reader, Tun, Writer};
use std::ffi::OsString;
use std::time::Duration;

/// Platform-specific extensions for the abstract device.
pub trait AbstractDeviceExt: AbstractDevice {
    fn tun_luid(&self) -> u64;
}

/// Windows-only interface configuration.
#[derive(Clone, Debug)]
pub struct PlatformConfig {
    pub(crate) device_guid: Option<u128>,
    pub(crate) wait_for_ipv4_interface: bool,
    pub(crate) wait_for_ipv6_interface: bool,
    pub(crate) wait_for_interface_timeout: Duration,
    pub(crate) wintun_file: OsString,
    pub(crate) dns_servers: Option<Vec<std::net::IpAddr>>,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            device_guid: None,
            wait_for_ipv4_interface: true,
            wait_for_ipv6_interface: true,
            wait_for_interface_timeout: Duration::from_secs(5),
            wintun_file: "wintun.dll".into(),
            dns_servers: None,
        }
    }
}

impl PlatformConfig {
    pub fn device_guid(&mut self, device_guid: u128) {
        log::trace!("Windows configuration device GUID");
        self.device_guid = Some(device_guid);
    }

    /// Use a custom path to the wintun.dll instead of looking in the working directory.
    /// Security note: It is up to the caller to ensure that the library can be safely loaded from
    /// the indicated path.
    ///
    /// [`wintun_file`](PlatformConfig::wintun_file) likes "path/to/wintun" or "path/to/wintun.dll".
    pub fn wintun_file<S: Into<OsString>>(&mut self, wintun_file: S) {
        self.wintun_file = wintun_file.into();
    }

    pub fn dns_servers(&mut self, dns_servers: &[std::net::IpAddr]) {
        self.dns_servers = Some(dns_servers.to_vec());
    }

    pub fn wait_for_interfaces(&mut self, ipv4: bool, ipv6: bool, timeout: Duration) {
        self.wait_for_ipv4_interface = ipv4;
        self.wait_for_ipv6_interface = ipv6;
        self.wait_for_interface_timeout = timeout;
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
