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

use bytes::BytesMut;
use futures::StreamExt;
use packet::{ip::Packet, Error};
use tokio::sync::mpsc::Receiver;
use tokio_util::codec::{Decoder, FramedRead};
use tun2::BoxError;

pub struct IPPacketCodec;

impl Decoder for IPPacketCodec {
    type Item = Packet<BytesMut>;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if buf.is_empty() {
            return Ok(None);
        }

        let buf = buf.split_to(buf.len());
        Ok(match Packet::no_payload(buf) {
            Ok(pkt) => Some(pkt),
            Err(err) => {
                println!("error {:?}", err);
                None
            }
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

    ctrlc2::set_async_handler(async move {
        tx.send(()).await.expect("Signal error");
    })
    .await;

    main_entry(rx).await?;
    Ok(())
}

async fn main_entry(mut quit: Receiver<()>) -> Result<(), BoxError> {
    let mut config = tun2::Configuration::default();

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
        config.device_guid(Some(9099482345783245345345_u128));
    });

    let dev = tun2::create_as_async(&config)?;

    let mut stream = FramedRead::new(dev, IPPacketCodec);

    loop {
        tokio::select! {
            _ = quit.recv() => {
                println!("Quit...");
                break;
            }
            Some(packet) = stream.next() => {
                match packet {
                    Ok(pkt) => println!("pkt: {:#?}", pkt),
                    Err(err) => panic!("Error: {:?}", err),
                }
            }
        };
    }
    Ok(())
}
