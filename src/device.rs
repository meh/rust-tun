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

use std::io::{Read, Write};
use std::net::Ipv4Addr;

use error::Error;
use configuration::Configuration;

/// A TUN device.
pub trait Device: Read + Write {
	/// Reconfigure the device.
	fn configure(&mut self, config: &Configuration) -> Result<(), Error> {
		if let Some(ip) = config.address {
			self.set_address(ip)?;
		}

		if let Some(ip) = config.destination {
			self.set_destination(ip)?;
		}

		if let Some(ip) = config.broadcast {
			self.set_broadcast(ip)?;
		}

		if let Some(ip) = config.netmask {
			self.set_netmask(ip)?;
		}

		if let Some(mtu) = config.mtu {
			self.set_mtu(mtu)?;
		}

		self.enabled(config.enabled)?;

		Ok(())
	}

	/// Get the device name.
	fn name(&self) -> &str;

	/// Set the device name.
	fn set_name(&mut self, name: &str) -> Result<(), Error>;

	/// Turn on or off the interface.
	fn enabled(&mut self, value: bool) -> Result<(), Error>;

	/// Get the address.
	fn address(&self) -> Result<Ipv4Addr, Error>;

	/// Set the address.
	fn set_address(&mut self, value: Ipv4Addr) -> Result<(), Error>;

	/// Get the destination address.
	fn destination(&self) -> Result<Ipv4Addr, Error>;

	/// Set the destination address.
	fn set_destination(&mut self, value: Ipv4Addr) -> Result<(), Error>;

	/// Get the broadcast address.
	fn broadcast(&self) -> Result<Ipv4Addr, Error>;

	/// Set the broadcast address.
	fn set_broadcast(&mut self, value: Ipv4Addr) -> Result<(), Error>;

	/// Get the netmask.
	fn netmask(&self) -> Result<Ipv4Addr, Error>;

	/// Set the netmask.
	fn set_netmask(&mut self, value: Ipv4Addr) -> Result<(), Error>;

	/// Get the MTU.
	fn mtu(&self) -> Result<i32, Error>;

	/// Set the MTU.
	fn set_mtu(&mut self, value: i32) -> Result<(), Error>;
}
