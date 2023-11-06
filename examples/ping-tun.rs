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

use futures::{SinkExt, StreamExt};
use packet::{builder::Builder, icmp, ip, Packet};
use tun::{self, Configuration, TunPacket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Configuration::default();

    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform(|config| {
        config.packet_information(true);
    });

    #[cfg(target_os = "windows")]
    config.platform(|config| {
        config.initialize();
    });

    let dev = tun::create_as_async(&config)?;

    let mut framed = dev.into_framed();

    while let Some(packet) = framed.next().await {
        let pkt = packet?;
        match ip::Packet::new(pkt.get_bytes()) {
            Ok(ip::Packet::V4(pkt)) => {
                if let Ok(icmp) = icmp::Packet::new(pkt.payload()) {
                    if let Ok(icmp) = icmp.echo() {
                        println!("{:?} - {:?}", icmp.sequence(), pkt.destination());
                        let reply = ip::v4::Builder::default()
                            .id(0x42)?
                            .ttl(64)?
                            .source(pkt.destination())?
                            .destination(pkt.source())?
                            .icmp()?
                            .echo()?
                            .reply()?
                            .identifier(icmp.identifier())?
                            .sequence(icmp.sequence())?
                            .payload(icmp.payload())?
                            .build()?;
                        framed.send(TunPacket::new(reply)).await?;
                    }
                }
            }
            Err(err) => println!("Received an invalid packet: {:?}", err),
            _ => {}
        }
    }
    Ok(())
}
