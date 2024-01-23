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

use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() {
    const MTU: i32 = 1500;
    let mut config = tun2::Configuration::default();

    config
        .address((10, 0, 0, 1))
        .netmask((255, 255, 255, 0))
        .mtu(MTU)
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.packet_information(true);
        config.apply_settings(true);
    });

    let mut dev = tun2::create_as_async(&config).unwrap();
    let mut buf: [u8; 1504] = [0u8; MTU as usize + 4];
    loop {
        match dev.read(&mut buf).await {
            Ok(len) => println!("pkt: {:?}", &buf[..len]),
            Err(_) => {}
        }
    }
}
