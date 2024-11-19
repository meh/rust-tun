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

#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub(crate) mod linux;
#[cfg(all(target_os = "linux", not(target_env = "ohos")))]
pub use self::linux::{create, Device, PlatformConfig};

#[cfg(target_os = "freebsd")]
pub(crate) mod freebsd;
#[cfg(target_os = "freebsd")]
pub use self::freebsd::{create, Device, PlatformConfig};

#[cfg(target_os = "macos")]
pub(crate) mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::{create, Device, PlatformConfig};

#[cfg(target_os = "ios")]
pub(crate) mod ios;
#[cfg(target_os = "ios")]
pub use self::ios::{create, Device, PlatformConfig};

#[cfg(target_os = "android")]
pub(crate) mod android;
#[cfg(target_os = "android")]
pub use self::android::{create, Device, PlatformConfig};

// Tip: OpenHarmony is a kind of Linux.
#[cfg(target_env = "ohos")]
pub(crate) mod ohos;
#[cfg(target_env = "ohos")]
pub use self::ohos::{create, Device, PlatformConfig};

#[cfg(unix)]
pub use crate::platform::posix::Tun;

#[cfg(target_os = "windows")]
pub(crate) mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::{create, Device, PlatformConfig, Tun};

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
}
