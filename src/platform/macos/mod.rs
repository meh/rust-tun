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

//! macOS specific functionality.

pub mod sys;

mod device;
pub use self::device::Device;

use crate::configuration::Configuration;
use crate::error::Result;

/// macOS-only interface configuration.
#[derive(Copy, Clone, Debug)]
pub struct PlatformConfig {
    pub(crate) packet_information: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        PlatformConfig {
            packet_information: true, // default is true in macOS
        }
    }
}

impl PlatformConfig {
    /// Enable or disable packet information, the first 4 bytes of
    /// each packet delivered from/to macOS underlying API is a header with flags and protocol type when enabled.
    ///
    /// If we open an `utun` device, there always exist PI.
    ///
    /// If we use `Network Extension` to build our App:
    ///
    /// If get the fd from
    /// ```Objective-C
    /// int32_t tunFd = [[NEPacketTunnelProvider.packetFlow valueForKeyPath:@"socket.fileDescriptor"] intValue];
    /// ```
    /// there exist PI.
    /// But if get packet from `[NEPacketTunnelProvider.packetFlow readPacketsWithCompletionHandler:]`, there is no PI.
    /// and when write packet via `[NEPacketTunnelProvider.packetFlow writePackets:withProtocols:]`, there is no PI either.
    pub fn packet_information(&mut self, value: bool) -> &mut Self {
        self.packet_information = value;
        self
    }
}

/// Create a TUN device with the given name.
pub fn create(configuration: &Configuration) -> Result<Device> {
    Device::new(configuration)
}
