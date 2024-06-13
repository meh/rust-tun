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

pub use device::{Device, Tun};

use crate::configuration::Configuration;
use crate::error::Result;

/// Windows-only interface configuration.
#[derive(Clone, Debug)]
pub struct PlatformConfig {
    pub(crate) device_guid: Option<u128>,
    pub(crate) wintun_path: String,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            device_guid: None,
            wintun_path: "wintun".to_string(),
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
    pub fn custom_wintun_path(&mut self, wintun_path: &str) {
        self.wintun_path = wintun_path.to_string();
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
