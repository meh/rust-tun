use std::io::Read;

extern crate tun;
use tun::Device;

fn main() {
	let mut dev = tun::create("fuffa0").unwrap();

	dev.set_address("10.0.0.1".parse().unwrap()).unwrap();
	dev.set_netmask("255.255.255.0".parse().unwrap()).unwrap();
	dev.enabled(true).unwrap();

	let mut buf = [0; 4096];

	loop {
		let amount = dev.read(&mut buf).unwrap();
		println!("{:?}", &buf[0 .. amount]);
	}
}
