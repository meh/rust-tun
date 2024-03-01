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

use packet::{builder::Builder, icmp, ip, Packet};
use std::io::{Read, Write};
use std::{net::Ipv4Addr, sync::mpsc::Receiver};
use tun2::{AbstractDevice, BoxError};

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
    let mut config = tun2::Configuration::default();

    config
        .tun_name("tun3")
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let mut dev = tun2::create(&config)?;
    let r = dev.address()?;
    println!("{:?}", r);

    let r = dev.destination()?;
    println!("{:?}", r);

    let r = dev.netmask()?;
    println!("{:?}", r);

    dev.set_address(std::net::IpAddr::V4(Ipv4Addr::new(10, 0, 0, 20)))?;
    dev.set_destination(std::net::IpAddr::V4(Ipv4Addr::new(10, 0, 0, 66)))?;
    dev.set_netmask(std::net::IpAddr::V4(Ipv4Addr::new(255, 255, 0, 0)))?;

    //dev.set_tun_name("tun8")?;

    //dev.set_address(std::net::IpAddr::V4(Ipv4Addr::new(10, 0, 0, 21)))?;

    // let r = dev.broadcast()?;
    // println!("{:?}",r);

    let mut buf = [0; 4096];

    std::thread::spawn(move || {
        loop {
            let amount = dev.read(&mut buf)?;
            let pkt = &buf[0..amount];
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
                            let size = dev.write(&reply[..])?;
                            println!("write {size} len {}", reply.len());
                        }
                    }
                }
                Err(err) => println!("Received an invalid packet: {:?}", err),
                _ => {}
            }
        }
        #[allow(unreachable_code)]
        Ok::<(), BoxError>(())
    });

    quit.recv().expect("Quit error.");
    Ok(())
}
