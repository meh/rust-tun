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
use tun2::AbstractDevice;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = tun2::Configuration::default();

    config
        .address((10, 0, 0, 1))
        .netmask((255, 255, 255, 0))
        .mtu(tun2::DEFAULT_MTU)
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.packet_information(true);
        config.ask_permission(true);
    });

    let mut dev = tun2::create_as_async(&config)?;
    let size = dev.as_ref().mtu()? + tun2::PACKET_INFORMATION_LENGTH;
    let mut buf = vec![0; size];
    loop {
        if let Ok(len) = dev.read(&mut buf).await {
            println!("pkt: {:?}", &buf[..len]);
        }
    }
    #[allow(unreachable_code)]
    Ok(())
}
