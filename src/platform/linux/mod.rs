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

//! Linux specific functionality.

pub mod sys;

mod device;
pub use self::device::Device;

use crate::configuration::Configuration;
use crate::error::Result;

/// Linux-only interface configuration.
#[derive(Copy, Clone, Debug)]
pub struct PlatformConfig {
    /// switch of Enable/Disable packet information for network driver
    pub(crate) packet_information: bool,
    /// root privileges required or not
    pub(crate) ensure_root_privileges: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        PlatformConfig {
            packet_information: false,
            ensure_root_privileges: true,
        }
    }
}

impl PlatformConfig {
    /// Enable or disable packet information, when enabled the first 4 bytes of
    /// each packet is a header with flags and protocol type.
    pub fn packet_information(&mut self, value: bool) -> &mut Self {
        self.packet_information = value;
        self
    }

    /// Indicated whether tun2 running in root privilege,
    /// since some operations need it such as assigning IP/netmask/destination etc.
    pub fn ensure_root_privileges(&mut self, value: bool) -> &mut Self {
        self.ensure_root_privileges = value;
        self
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
