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

extern crate tun;
use tun::Device;

fn main() {
	let mut dev = tun::configure("fuffa0").unwrap()
		.address((10, 0, 0, 1)).unwrap()
		.netmask((255, 255, 255, 0)).unwrap()
		.up().unwrap();

	let mut buf = [0; 4096];

	loop {
		let amount = dev.read(&mut buf).unwrap();
		println!("{:?}", &buf[0 .. amount]);
	}
}
