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

//! Platform specific modules.

#[cfg(unix)]
pub(crate) mod posix;
#[cfg(unix)]
pub use posix::{Reader, Writer};

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub(crate) mod linux;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub use self::linux::{Device, PlatformConfig, create};

#[cfg(target_os = "freebsd")]
pub(crate) mod freebsd;
#[cfg(target_os = "freebsd")]
pub use self::freebsd::{Device, PlatformConfig, create};

#[cfg(target_os = "macos")]
pub(crate) mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::{Device, PlatformConfig, create};

#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub(crate) mod ios;
#[cfg(any(target_os = "ios", target_os = "tvos"))]
pub use self::ios::{Device, PlatformConfig, create};

#[cfg(target_os = "android")]
pub(crate) mod android;
#[cfg(target_os = "android")]
pub use self::android::{Device, PlatformConfig, create};

// Tip: OpenHarmony is a kind of Linux.
#[cfg(target_env = "ohos")]
pub(crate) mod ohos;
#[cfg(target_env = "ohos")]
pub use self::ohos::{Device, PlatformConfig, create};

#[cfg(target_os = "windows")]
pub(crate) mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::{AbstractDeviceExt, Device, PlatformConfig, Reader, Tun, Writer, create};

#[cfg(test)]
mod test {
    use crate::configuration::Configuration;
    use crate::device::AbstractDevice;
    use std::net::Ipv4Addr;

    #[test]
    fn create() {
        let dev = super::create(
            Configuration::default()
                .tun_name("utun6")
                .address("192.168.50.1")
                .netmask("255.255.0.0")
                .mtu(crate::DEFAULT_MTU)
                .up(),
        )
        .unwrap();

        assert_eq!(
            "192.168.50.1".parse::<Ipv4Addr>().unwrap(),
            dev.address().unwrap()
        );

        assert_eq!(
            "255.255.0.0".parse::<Ipv4Addr>().unwrap(),
            dev.netmask().unwrap()
        );

        assert_eq!(crate::DEFAULT_MTU, dev.mtu().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn recv_timeout_times_out_on_a_silent_device() {
        use std::time::{Duration, Instant};

        // The device is never brought up, so no traffic can reach it.
        let config = Configuration::default();
        let dev = super::create(&config).unwrap();
        let mut buf = [0u8; crate::DEFAULT_MTU as usize];
        let timeout = Duration::from_millis(100);
        let start = Instant::now();
        let err = dev.recv_timeout(&mut buf, timeout).unwrap_err();
        let elapsed = start.elapsed();
        assert_eq!(err.kind(), std::io::ErrorKind::TimedOut);
        assert!(elapsed >= timeout, "returned after {elapsed:?}");
        assert!(elapsed < Duration::from_secs(10), "took {elapsed:?}");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn recv_timeout_returns_a_packet_arriving_before_expiry() {
        use std::time::Duration;

        let dev = super::create(
            Configuration::default()
                .address("192.168.51.1")
                .netmask("255.255.255.0")
                .up(),
        )
        .unwrap();

        // Route a packet into the device from the kernel side; it stays
        // queued on the interface until the read below picks it up.
        let socket = std::net::UdpSocket::bind("192.168.51.1:0").unwrap();
        socket.send_to(b"ping", "192.168.51.2:9").unwrap();

        let mut buf = [0u8; crate::DEFAULT_MTU as usize];
        let amount = dev.recv_timeout(&mut buf, Duration::from_secs(5)).unwrap();
        assert!(amount > 0);
    }
}
