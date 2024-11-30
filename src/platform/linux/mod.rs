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

mod sys;

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

    /// Enable IFF_NAPI
    pub(crate) napi: bool,

    /// Enable IFF_VNET_HDR
    pub(crate) vnet_hdr: bool,
}

/// `packet_information` is default to be `false` and `ensure_root_privileges` is default to be `true`.
impl Default for PlatformConfig {
    fn default() -> Self {
        PlatformConfig {
            packet_information: false,
            ensure_root_privileges: true,
            napi: false,
            vnet_hdr: false,
        }
    }
}

impl PlatformConfig {
    /// Enable or disable packet information, the first 4 bytes of
    /// each packet delivered from/to Linux underlying API is a header with flags and protocol type when enabled.
    ///
    /// [Note: This configuration just applies to the Linux underlying API and is a no-op on `tun` crate
    /// (i.e. the packets delivered from/to `tun` crate must always NOT contain packet information) -- end note].
    #[deprecated(
        since = "0.7.0",
        note = "No effect applies to the packets delivered from/to tun since the packets always contain no header on all platforms."
    )]
    pub fn packet_information(&mut self, value: bool) -> &mut Self {
        self.packet_information = value;
        self
    }

    /// Indicated whether tun running in root privilege,
    /// since some operations need it such as assigning IP/netmask/destination etc.
    pub fn ensure_root_privileges(&mut self, value: bool) -> &mut Self {
        self.ensure_root_privileges = value;
        self
    }

    /// Enable / Disable IFF_NAPI flag.
    pub fn napi(&mut self, value: bool) -> &mut Self {
        self.napi = value;
        self
    }

    /// Enable / Disable IFF_VNET_HDR flag.
    pub fn vnet_hdr(&mut self, value: bool) -> &mut Self {
        self.vnet_hdr = value;
        self
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
