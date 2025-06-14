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

// You can test this example with command `ping 10.0.3.1`.

use packet::{Packet, builder::Builder, icmp, ip};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tun::BoxError;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let cancel_token = tokio_util::sync::CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    let ctrlc = ctrlc2::AsyncCtrlC::new(move || {
        cancel_token_clone.cancel();
        true
    })?;

    main_entry(cancel_token).await?;
    ctrlc.await?;
    Ok(())
}

async fn main_entry(cancel_token: tokio_util::sync::CancellationToken) -> Result<(), BoxError> {
    let mut config = tun::Configuration::default();

    config
        .address((10, 0, 3, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 3, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let dev = tun::create_as_async(&config)?;
    let (mut writer, mut reader) = dev.split()?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let t1_token = cancel_token.clone();
    let t2_token = cancel_token.clone();

    let t1 = tokio::spawn(async move {
        let mut buf = [0; 4096];
        loop {
            let size = tokio::select! {
                _ = t1_token.cancelled() => break,
                res = reader.read(&mut buf) => res?,
            };
            let pkt = &buf[..size];
            tx.send(pkt.to_vec()).await.map_err(std::io::Error::other)?;
        }
        Ok::<(), std::io::Error>(())
    });

    let t2 = tokio::spawn(async move {
        loop {
            let pkt = tokio::select! {
                _ = t2_token.cancelled() => break,
                opt = rx.recv() => opt.ok_or(packet::Error::Io(std::io::Error::other("Channel closed")))?,
            };
            match ip::Packet::new(pkt.as_slice()) {
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
                            writer.write_all(&reply[..]).await?;
                        }
                    }
                }
                Err(err) => println!("Received an invalid packet: {:?}", err),
                _ => println!("receive pkt {:?}", pkt),
            }
        }
        Ok::<(), packet::Error>(())
    });
    let v = tokio::join!(t1, t2);
    println!("Exiting... {v:?}");
    Ok(())
}
