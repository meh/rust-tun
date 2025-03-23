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

//! OpenHarmony specific functionality.
#![allow(unused_variables)]
#![allow(dead_code)]
mod device;
pub use self::device::Device;

use crate::configuration::Configuration;
use crate::error::Result;

/// OpenHarmony-only interface configuration.
#[derive(Copy, Clone, Default, Debug)]
pub struct PlatformConfig;

impl PlatformConfig {
    /// Dummy functions for compatibility with Linux.
    pub fn packet_information(&mut self, _value: bool) -> &mut Self {
        self
    }

    /// Dummy functions for compatibility with Linux.
    pub fn ensure_root_privileges(&mut self, _value: bool) -> &mut Self {
        self
    }

    /// Dummy functions for compatibility with Linux.
    pub fn napi(&mut self, _value: bool) -> &mut Self {
        self
    }

    /// Dummy functions for compatibility with Linux.
    pub fn vnet_hdr(&mut self, _value: bool) -> &mut Self {
        self
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
