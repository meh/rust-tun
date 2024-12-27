TUN interfaces 
==============
[![Crates.io](https://img.shields.io/crates/v/tun.svg)](https://crates.io/crates/tun)
[![tun](https://docs.rs/tun/badge.svg)](https://docs.rs/tun/latest/tun/)
![WTFPL](http://img.shields.io/badge/license-WTFPL-blue.svg)

This crate allows the creation and usage of TUN interfaces, the aim is to make this cross-platform.

Usage
-----
First, add the following to your `Cargo.toml`:

```toml
[dependencies]
tun = "0.7"
```

If you want to use the TUN interface with mio/tokio, you need to enable the `async` feature:

```toml
[dependencies]
tun = { version = "0.7", features = ["async"] }
```

Example
-------
The following example creates and configures a TUN interface and starts reading
packets from it.

```rust
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut config = tun::Configuration::default();
    config
        .address((10, 0, 0, 9))
        .netmask((255, 255, 255, 0))
        .destination((10, 0, 0, 1))
        .up();

    #[cfg(target_os = "linux")]
    config.platform_config(|config| {
        // requiring root privilege to acquire complete functions
        config.ensure_root_privileges(true);
    });

    let mut dev = tun::create(&config)?;
    let mut buf = [0; 4096];

    loop {
        let amount = dev.read(&mut buf)?;
        println!("{:?}", &buf[0..amount]);
    }
}
```

Platforms
=========
## Supported Platforms

- [x] Windows
- [x] Linux
- [x] macOS
- [x] FreeBSD
- [x] Android
- [x] iOS
- [x] OpenHarmony


Linux
-----
You will need the `tun` module to be loaded and root is required to create
interfaces.

macOS & FreeBSD
-----
`tun` will automatically set up a route according to the provided configuration, which does a similar thing like this:
> sudo route -n add -net 10.0.0.0/24 10.0.0.1


iOS
----
You can pass the file descriptor of the TUN device to `tun` to create the interface.

Here is an example to create the TUN device on iOS and pass the `fd` to `tun`:
```swift
// Swift
class PacketTunnelProvider: NEPacketTunnelProvider {
    override func startTunnel(options: [String : NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        let tunnelNetworkSettings = createTunnelSettings() // Configure TUN address, DNS, mtu, routing...
        setTunnelNetworkSettings(tunnelNetworkSettings) { [weak self] error in
            // The tunnel of this tunFd is contains `Packet Information` prifix.
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
        let mut cfg = tun::Configuration::default();
        cfg.raw_fd(fd);
        #[cfg(target_os = "ios")]
        cfg.platform_config(|p_cfg| {
            p_cfg.packet_information(true);
        });
        let mut tun = tun::create_as_async(&cfg).unwrap();
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
the same directory as your executable and run your program as administrator.


OpenHarmony
-----
You can pass the file descriptor of the TUN device to `tun` to create the interface. You can see the detail [VPN document](https://developer.huawei.com/consumer/en/doc/harmonyos-references-V5/js-apis-net-vpnextension-V5).

Here is an example to create the TUN device on OpenHarmony/HarmonyNext and pass the `fd` to `tun`:
```ts
// ArkTS
import vpnExtension from '@ohos.net.vpnExtension';
import vpnClient from 'libvpn_client.so';

const VpnConnection: vpnExtension.VpnConnection = vpnExtension.createVpnConnection(this.context);

async function setup() {
    const fd = await VpnConnection.create(config);
    vpnClient.setup(fd);
}
```

```rust
// use ohos-rs to bind rust for arkts
use napi_derive_ohos::napi;

#[napi]
async fn setup(fd: i32) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let mut cfg = tun::Configuration::default();
        cfg.raw_fd(fd);
        let mut tun = tun::create_as_async(&cfg).unwrap();
        let mut framed = tun.into_framed();
        while let Some(packet) = framed.next().await {
            ...
        }
    });
}
```

## Contributors ✨
Thanks goes to these wonderful people:

<a href="https://github.com/meh/rust-tun/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=meh/rust-tun" />
</a>
