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
use std::io::{Read, Write};
use std::sync::mpsc::Receiver;
use tun::BoxError;

fn main() -> Result<(), BoxError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = ctrlc2::set_handler(move || {
        tx.send(()).expect("Signal error.");
        true
    })
    .expect("Error setting Ctrl-C handler");

    main_entry(rx)?;
    handle.join().unwrap();
    Ok(())
}

fn main_entry(quit: Receiver<()>) -> Result<(), BoxError> {
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

    let dev = tun::create(&config)?;
    let (mut reader, mut writer) = dev.split();

    let (tx, rx) = std::sync::mpsc::channel();

    let _t1 = std::thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            let size = reader.read(&mut buf)?;
            let pkt = &buf[..size];
            tx.send(pkt.to_vec()).map_err(std::io::Error::other)?;
        }
        #[allow(unreachable_code)]
        Ok::<(), std::io::Error>(())
    });

    let _t2 = std::thread::spawn(move || {
        loop {
            if let Ok(pkt) = rx.recv() {
                match ip::Packet::new(pkt.as_slice()) {
                    Ok(ip::Packet::V4(pkt)) => {
                        if let Ok(icmp) = icmp::Packet::new(pkt.payload())
                            && let Ok(icmp) = icmp.echo()
                        {
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
                            writer.write_all(&reply[..])?;
                        }
                    }
                    Err(err) => println!("Received an invalid packet: {err:?}"),
                    _ => {
                        println!("receive pkt {pkt:?}");
                    }
                }
            }
        }
        #[allow(unreachable_code)]
        Ok::<(), packet::Error>(())
    });
    quit.recv().expect("Quit error.");
    Ok(())
}
