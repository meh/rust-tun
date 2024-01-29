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

use std::io::Read;
use std::sync::mpsc::Receiver;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

fn main_entry(quit: Receiver<()>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    let mut dev = tun2::create(&config)?;
    std::thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            let amount = dev.read(&mut buf)?;
            println!("{:?}", &buf[0..amount]);
        }
        #[allow(unreachable_code)]
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });
    quit.recv().expect("Quit error.");
    Ok(())
}
