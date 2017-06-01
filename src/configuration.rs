//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE Version 2, December 2004
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

use std::net::{Ipv4Addr};

use error::*;
use device::Device;
use address::IntoAddress;
use platform;

#[derive(Clone, Default, Debug)]
pub struct Configuration {
	pub(crate) name:     Option<String>,
	pub(crate) platform: platform::Configuration,

	pub(crate) address:     Option<Ipv4Addr>,
	pub(crate) destination: Option<Ipv4Addr>,
	pub(crate) broadcast:   Option<Ipv4Addr>,
	pub(crate) netmask:     Option<Ipv4Addr>,
	pub(crate) mtu:         Option<i32>,
	pub(crate) enabled:     bool,
}

impl Configuration {
	pub fn platform<T, F>(&mut self, f: F) -> &mut Self
		where F: FnOnce(&mut platform::Configuration)
	{
		f(&mut self.platform);
		self
	}

	pub fn name<S: AsRef<str>>(&mut self, name: S) -> &mut Self {
		self.name = Some(name.as_ref().into());
		self
	}

	pub fn address<A: IntoAddress>(&mut self, value: A) -> &mut Self {
		self.address = Some(value.into_address().unwrap());
		self
	}

	pub fn destination<A: IntoAddress>(&mut self, value: A) -> &mut Self {
		self.destination = Some(value.into_address().unwrap());
		self
	}

	pub fn broadcast<A: IntoAddress>(&mut self, value: A) -> &mut Self {
		self.broadcast = Some(value.into_address().unwrap());
		self
	}

	pub fn netmask<A: IntoAddress>(&mut self, value: A) -> &mut Self {
		self.netmask = Some(value.into_address().unwrap());
		self
	}

	pub fn mtu(&mut self, value: i32) -> &mut Self {
		self.mtu = Some(value);
		self
	}

	pub fn up(&mut self) -> &mut Self {
		self.enabled = true;
		self
	}

	pub fn down(&mut self) -> &mut Self {
		self.enabled = false;
		self
	}

	pub fn apply<T: Device>(&self, dev: &mut T) -> Result<()> {
		if let Some(ip) = self.address {
			dev.set_address(ip)?;
		}

		if let Some(ip) = self.destination {
			dev.set_address(ip)?;
		}

		if let Some(ip) = self.broadcast {
			dev.set_broadcast(ip)?;
		}

		if let Some(ip) = self.netmask {
			dev.set_netmask(ip)?;
		}

		if let Some(mtu) = self.mtu {
			dev.set_mtu(mtu)?;
		}

		dev.enabled(self.enabled)?;

		Ok(())
	}
}
