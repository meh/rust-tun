mod sys;

mod device;
pub use self::device::Device;

use error::*;
use configuration::Configuration as C;

#[derive(Copy, Clone, Default, Debug)]
pub struct Configuration {
	pub(crate) packet_information: bool,
}

impl Configuration {
	pub fn packet_information(&mut self, value: bool) -> &mut Self {
		self.packet_information = value;
		self
	}
}

/// Create a TUN device with the given name.
pub fn create(configuration: &C) -> Result<Device> {
	Device::new(&configuration)
}
