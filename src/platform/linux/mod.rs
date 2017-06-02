//! Linux specific functionality.

pub mod sys;

mod device;
pub use self::device::Device;

use error::*;
use configuration::Configuration as C;

/// Linux-only interface configuration.
#[derive(Copy, Clone, Default, Debug)]
pub struct Configuration {
	pub(crate) packet_information: bool,
}

impl Configuration {
	/// Enable or disable packet information, when enabled the first 4 bytes of
	/// each packet is a header with flags and protocol type.
	pub fn packet_information(&mut self, value: bool) -> &mut Self {
		self.packet_information = value;
		self
	}
}

/// Create a TUN device with the given name.
pub fn create(configuration: &C) -> Result<Device> {
	Device::new(&configuration)
}
