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
use tokio::sync::mpsc::Receiver;
use tun2::{self, Configuration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

    ctrlc2::set_async_handler(async move {
        tx.send(()).await.expect("Signal error");
    })
    .await;

    main_entry(rx).await?;
    Ok(())
}

async fn main_entry(
    mut quit: Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut config = Configuration::default();

    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.packet_information(true);
        config.ask_permission(true);
    });

    #[cfg(target_os = "windows")]
    config.platform_config(|config| {
        config.device_guid(Some(9099482345783245345345_u128));
    });

    let dev = tun2::create_as_async(&config)?;

    let mut framed = dev.into_framed();

    loop {
        tokio::select! {
            _ = quit.recv() => {
                println!("Quit...");
                break;
            }
            Some(packet) = framed.next() => {
                let pkt: Vec<u8> = packet?;
                match ip::Packet::new(pkt) {
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
                                framed.send(reply).await?;
                            }
                        }
                    }
                    Err(err) => println!("Received an invalid packet: {:?}", err),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
