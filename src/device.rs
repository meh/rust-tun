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

use crate::configuration::Configuration;
use crate::error::Result;
use std::io::{Read, Write};
use std::net::IpAddr;

/// A TUN abstract device interface.
pub trait AbstractDevice: Read + Write {
    /// Reconfigure the device.
    fn configure(&mut self, config: &Configuration) -> Result<()> {
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

        if let Some(enabled) = config.enabled {
            self.enabled(enabled)?;
        }

        Ok(())
    }

    /// Get the device index.
    fn tun_index(&self) -> Result<i32>;

    /// Get the device tun name.
    fn tun_name(&self) -> Result<String>;

    /// Set the device tun name.
    fn set_tun_name(&mut self, tun_name: &str) -> Result<()>;

    /// Turn on or off the interface.
    fn enabled(&mut self, value: bool) -> Result<()>;

    /// Get the address.
    fn address(&self) -> Result<IpAddr>;

    /// Set the address.
    fn set_address(&mut self, value: IpAddr) -> Result<()>;

    /// Get the destination address.
    fn destination(&self) -> Result<IpAddr>;

    /// Set the destination address.
    fn set_destination(&mut self, value: IpAddr) -> Result<()>;

    /// Get the broadcast address.
    fn broadcast(&self) -> Result<IpAddr>;

    /// Set the broadcast address.
    fn set_broadcast(&mut self, value: IpAddr) -> Result<()>;

    /// Get the netmask.
    fn netmask(&self) -> Result<IpAddr>;

    /// Set the netmask.
    fn set_netmask(&mut self, value: IpAddr) -> Result<()>;

    /// Get the MTU.
    fn mtu(&self) -> Result<u16>;

    /// Set the MTU.
    ///
    /// [Note: This setting has no effect on the Windows platform due to the mtu of wintun is always 65535. --end note]
    fn set_mtu(&mut self, value: u16) -> Result<()>;

    /// Return whether the underlying tun device on the platform has packet information
    ///
    /// [Note: This value is not used to specify whether the packets delivered from/to tun have packet information. -- end note]
    fn packet_information(&self) -> bool;
}
