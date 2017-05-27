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
	inner: Option<T>,
}

impl<T: Device> Configuration<T> {
	pub fn new(inner: T) -> Self {
		Configuration {
			inner: Some(inner)
		}
	}

	pub fn name<S: AsRef<str>>(&mut self, name: S) -> Result<&mut Self> {
		self.inner.as_mut().unwrap().set_name(name.as_ref())?;
		Ok(self)
	}

	pub fn address<A: IntoAddress>(&mut self, value: A) -> Result<&mut Self> {
		self.inner.as_mut().unwrap().set_address(value.into_address()?)?;
		Ok(self)
	}

	pub fn destination<A: IntoAddress>(&mut self, value: A) -> Result<&mut Self> {
		self.inner.as_mut().unwrap().set_destination(value.into_address()?)?;
		Ok(self)
	}

	pub fn broadcast<A: IntoAddress>(&mut self, value: A) -> Result<&mut Self> {
		self.inner.as_mut().unwrap().set_broadcast(value.into_address()?)?;
		Ok(self)
	}

	pub fn netmask<A: IntoAddress>(&mut self, value: A) -> Result<&mut Self> {
		self.inner.as_mut().unwrap().set_netmask(value.into_address()?)?;
		Ok(self)
	}

	pub fn device<F>(&mut self, f: F) -> Result<&mut Self>
		where F: FnOnce(&mut T) -> Result<()>
	{
		f(self.inner.as_mut().unwrap())?;
		Ok(self)
	}

	pub fn up(&mut self) -> Result<T> {
		self.inner.as_mut().unwrap().enabled(true)?;
		Ok(self.inner.take().unwrap())
	}

	pub fn down(&mut self) -> Result<T> {
		self.inner.as_mut().unwrap().enabled(false)?;
		Ok(self.inner.take().unwrap())
	}
}
