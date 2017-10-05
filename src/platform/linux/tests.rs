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

use std::io::Read;
use std::net::Ipv4Addr;
use std::str::FromStr;

use libc;

use create;
use configuration;
use device::Device;

#[test]
fn test_tun_create() {
	let mut config = configuration::Configuration::default();

	let addr = Ipv4Addr::from_str("192.168.50.1").unwrap();
	let netmask = Ipv4Addr::from_str("255.255.0.0").unwrap();
	let mtu = 1480;

	config.name("utun6")
		.address(addr)
		.netmask(netmask)
		.mtu(mtu)
		.up();

	let mut dev = create(&config).unwrap();

	let g_addr: Ipv4Addr = dev.address().unwrap().into();
	assert_eq!(addr, g_addr);

	let g_netmask: Ipv4Addr = dev.netmask().unwrap().into();
	assert_eq!(netmask, g_netmask);

	let g_mtu = dev.mtu().unwrap();
	assert_eq!(mtu, g_mtu);
}
