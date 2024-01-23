TUN interfaces 
==============
[![Crates.io](https://img.shields.io/crates/v/tun2.svg)](https://crates.io/crates/tun2)
![tun2](https://docs.rs/tun2/badge.svg)
![WTFPL](http://img.shields.io/badge/license-WTFPL-blue.svg)

This crate allows the creation and usage of TUN interfaces, the aim is to make this cross-platform.

> Since the original maintainer @meh is no longer interested in continuing to maintain
> [tun](https://crates.io/crates/tun) at [repo](https://github.com/meh/rust-tun),
> I (@ssrlive) created the [tun2](https://github.com/ssrlive/rust-tun) branch repo and
> continued to actively update. Welcome to any interested contributor.
> If you want to be a co-contributor and publisher of [tun2](https://crates.io/crates/tun2),
> please contact me in [issues](https://github.com/ssrlive/rust-tun/issues).
>
> For me, a submitted PR has not been reviewed for a long time,
> cannot be merged to the main branch, and cannot be published.
> It is like a patient who has not been sutured on the operating table for a long time.
> This is a bad experience.
> I believe that many people feel the same.

Usage
-----
First, add the following to your `Cargo.toml`:

```toml
[dependencies]
tun2 = "0.7"
```

If you want to use the TUN interface with mio/tokio, you need to enable the `async` feature:

```toml
[dependencies]
tun2 = { version = "0.7", features = ["async"] }
```

Example
-------
The following example creates and configures a TUN interface and starts reading
packets from it.

```rust
use std::io::Read;

fn main() {
    let mut config = tun2::Configuration::default();
    config
        .address((10, 0, 0, 1))
        .netmask((255, 255, 255, 0))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        config.packet_information(true);
        config.apply_settings(true);
    });

    let mut dev = tun2::create(&config).unwrap();
    let mut buf = [0; 4096];

    loop {
        let amount = dev.read(&mut buf).unwrap();
        println!("{:?}", &buf[0..amount]);
    }
}
```

Platforms
=========
Recently, `tun2` supports Linux, Android, macOS, iOS and Windows.

Linux
-----
You will need the `tun2` module to be loaded and root is required to create
interfaces.

macOS
-----
It just works, but you have to set up routing manually. For example:
> sudo route -n add -net 10.0.0.0/24 10.0.0.1


iOS
----
You can pass the file descriptor of the TUN device to `tun2` to create the interface.

Here is an example to create the TUN device on iOS and pass the `fd` to `tun2`:
```swift
// Swift
class PacketTunnelProvider: NEPacketTunnelProvider {
    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let tunnelNetworkSettings = createTunnelSettings() // Configure TUN address, DNS, mtu, routing...
        setTunnelNetworkSettings(tunnelNetworkSettings) { [weak self] error in
            let tunFd = self?.packetFlow.value(forKeyPath: "socket.fileDescriptor") as! Int32
            DispatchQueue.global(qos: .default).async {
                start_tun(tunFd)
            }
            completionHandler(nil)
        }
    }
}
```

```rust
#[no_mangle]
pub extern "C" fn start_tun(fd: std::os::raw::c_int) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut cfg = tun2::Configuration::default();
        cfg.raw_fd(fd);
        let mut tun = tun2::create_as_async(&cfg).unwrap();
        let mut framed = tun.into_framed();
        while let Some(packet) = framed.next().await {
            ...
        }
    });
}
```

Windows
-----
You need to copy the [wintun.dll](https://wintun.net/) file which matches your architecture to 
the same directory as your executable and run you program as administrator.
