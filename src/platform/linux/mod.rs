mod sys;
mod device;
pub use self::device::Device;

use error;

/// Create a TUN device with the given name.
pub fn create<S: AsRef<str>>(name: S) -> error::Result<Device> {
	Device::allocate(Some(name.as_ref()))
}

/// Create a TUN device with the next available name.
pub fn next() -> error::Result<Device> {
	Device::allocate(None)
}
