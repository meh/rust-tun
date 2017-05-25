mod sys;
mod device;
pub use self::device::Device;

use error::*;
use configuration::Configuration;

/// Create a TUN device with the given name.
pub fn create<S: AsRef<str>>(name: S) -> Result<Device> {
	Device::allocate(Some(name.as_ref()))
}

/// Create a TUN device with the next available name.
pub fn next() -> Result<Device> {
	Device::allocate(None)
}

pub fn configure<S: AsRef<str>>(name: S) -> Result<Configuration<Device>> {
	use device::Device;
	Ok(create(name)?.configure())
}
