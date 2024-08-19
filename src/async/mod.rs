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

//! Async specific modules.

use crate::error;

use crate::configuration::Configuration;
use crate::platform::create;

mod codec;
pub use codec::TunPacketCodec;

#[cfg(unix)]
mod unix_device;
#[cfg(unix)]
pub use unix_device::{AsyncDevice, DeviceReader, DeviceWriter};

#[cfg(target_os = "windows")]
mod win_device;
#[cfg(target_os = "windows")]
pub use win_device::{AsyncDevice, DeviceReader, DeviceWriter};

/// Create a TUN device with the given name.
pub fn create_as_async(configuration: &Configuration) -> Result<AsyncDevice, error::Error> {
    let device = create(configuration)?;
    AsyncDevice::new(device).map_err(|err| err.into())
}
