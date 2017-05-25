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

use error::*;
use device::Device;
use address::IntoAddress;

pub struct Configuration<T: Device> {
	inner: T,
}

impl<T: Device> Configuration<T> {
	pub fn new(inner: T) -> Self {
		Configuration {
			inner: inner
		}
	}

	pub fn name<S: AsRef<str>>(mut self, name: S) -> Result<Self> {
		self.inner.set_name(name.as_ref())?;
		Ok(self)
	}

	pub fn address<A: IntoAddress>(mut self, value: A) -> Result<Self> {
		self.inner.set_address(value.into_address()?)?;
		Ok(self)
	}

	pub fn destination<A: IntoAddress>(mut self, value: A) -> Result<Self> {
		self.inner.set_destination(value.into_address()?)?;
		Ok(self)
	}

	pub fn broadcast<A: IntoAddress>(mut self, value: A) -> Result<Self> {
		self.inner.set_broadcast(value.into_address()?)?;
		Ok(self)
	}

	pub fn netmask<A: IntoAddress>(mut self, value: A) -> Result<Self> {
		self.inner.set_netmask(value.into_address()?)?;
		Ok(self)
	}

	pub fn up(mut self) -> Result<T> {
		self.inner.enabled(true)?;
		Ok(self.inner)
	}

	pub fn down(mut self) -> Result<T> {
		self.inner.enabled(false)?;
		Ok(self.inner)
	}
}
