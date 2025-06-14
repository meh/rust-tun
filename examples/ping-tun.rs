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
use packet::{Packet, builder::Builder, icmp, ip};
use tokio_util::sync::CancellationToken;
use tun::{self, BoxError, Configuration};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let token = CancellationToken::new();
    let token_clone = token.clone();

    let ctrlc = ctrlc2::AsyncCtrlC::new(move || {
        token_clone.cancel();
        true
    })?;

    main_entry(token).await?;
    ctrlc.await?;
    Ok(())
}

async fn main_entry(token: CancellationToken) -> Result<(), BoxError> {
    let mut config = Configuration::default();

    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        #[allow(deprecated)]
        config.packet_information(true);
        config.ensure_root_privileges(true);
    });

    #[cfg(target_os = "windows")]
    config.platform_config(|config| {
        config.device_guid(9099482345783245345345_u128);
    });

    let dev = tun::create_as_async(&config)?;

    let mut framed = dev.into_framed();

    loop {
        tokio::select! {
            _ = token.cancelled() => {
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
