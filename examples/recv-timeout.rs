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

//! Reads from the device on a dedicated thread and shuts that thread down
//! cleanly, without routing a wake-up packet into the interface and without
//! leaking the device. `recv_timeout` bounds every read, so the thread
//! notices the stop flag within one timeout interval.

#[cfg(unix)]
fn main() -> Result<(), tun::BoxError> {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let mut config = tun::Configuration::default();

    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let dev = tun::create(&config)?;

    let stop = Arc::new(AtomicBool::new(false));
    let reader_stop = stop.clone();
    let reader = std::thread::spawn(move || -> std::io::Result<()> {
        let mut buf = [0; 4096];
        while !reader_stop.load(Ordering::Relaxed) {
            match dev.recv_timeout(&mut buf, Duration::from_millis(500)) {
                Ok(amount) => println!("{:?}", &buf[0..amount]),
                Err(err) if err.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(err) => return Err(err),
            }
        }
        Ok(())
    });

    println!("reading for 5 seconds, then asking the reader to stop");
    std::thread::sleep(Duration::from_secs(5));
    stop.store(true, Ordering::Relaxed);
    reader.join().unwrap()?;
    println!("reader thread stopped cleanly");
    Ok(())
}

#[cfg(not(unix))]
fn main() {
    println!("recv_timeout is not available on this platform");
}
