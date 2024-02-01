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
use tokio::sync::mpsc::Receiver;
use tun2::AbstractDevice;

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
    let mut config = tun2::Configuration::default();

    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .mtu(tun2::DEFAULT_MTU)
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.ensure_root_privileges(true);
    });

    let mut dev = tun2::create_as_async(&config)?;
    let size = dev.as_ref().mtu()? as usize + tun2::PACKET_INFORMATION_LENGTH;
    let mut buf = vec![0; size];
    loop {
        tokio::select! {
            _ = quit.recv() => {
                println!("Quit...");
                break;
            }
            len = dev.read(&mut buf) => {
                println!("pkt: {:?}", &buf[..len?]);
            }
        };
    }
    Ok(())
}
