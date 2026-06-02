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

use std::io;
use std::net::Ipv4Addr;

use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, NO_ERROR};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    InitializeUnicastIpAddressEntry, SetIpInterfaceEntry,
};

pub fn netmask_to_prefix_len(mask: Ipv4Addr) -> u8 {
    let bits = u32::from(mask);
    let prefix = bits.leading_ones() as u8;
    debug_assert_eq!(
        bits,
        u32::MAX.checked_shl(32 - prefix as u32).unwrap_or(0),
        "non-contiguous netmask"
    );
    prefix
}

pub fn set_unicast_address(luid: u64, address: Ipv4Addr, mask: Ipv4Addr) -> io::Result<()> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        CreateUnicastIpAddressEntry, DeleteUnicastIpAddressEntry, GetUnicastIpAddressEntry,
        MIB_UNICASTIPADDRESS_ROW,
    };
    use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
    use windows_sys::Win32::Networking::WinSock::AF_INET;

    unsafe {
        // For deletion: use minimal row with only Address + Interface identifier
        let mut probe_row: MIB_UNICASTIPADDRESS_ROW = std::mem::zeroed();
        InitializeUnicastIpAddressEntry(&mut probe_row);
        probe_row.InterfaceLuid = NET_LUID_LH { Value: luid };
        probe_row.Address.si_family = AF_INET;
        probe_row.Address.Ipv4.sin_family = AF_INET;
        probe_row.Address.Ipv4.sin_addr.S_un.S_addr = u32::from_ne_bytes(address.octets());

        match GetUnicastIpAddressEntry(&mut probe_row) {
            NO_ERROR => {
                let del_status = DeleteUnicastIpAddressEntry(&probe_row);
                if del_status != NO_ERROR {
                    log::warn!("DeleteUnicastIpAddressEntry failed: {del_status}");
                }
            }
            ERROR_NOT_FOUND => {}
            status => {
                log::warn!("GetUnicastIpAddressEntry probe failed: {status}");
            }
        }

        // For creation: initialize with defaults, then set required fields
        let mut create_row: MIB_UNICASTIPADDRESS_ROW = std::mem::zeroed();
        InitializeUnicastIpAddressEntry(&mut create_row);

        create_row.InterfaceLuid = NET_LUID_LH { Value: luid };
        create_row.Address.si_family = AF_INET;
        create_row.Address.Ipv4.sin_family = AF_INET;
        create_row.Address.Ipv4.sin_addr.S_un.S_addr = u32::from_ne_bytes(address.octets());
        create_row.OnLinkPrefixLength = netmask_to_prefix_len(mask);
        create_row.DadState = 4; // IpDadStatePreferred
        create_row.ValidLifetime = u32::MAX;
        create_row.PreferredLifetime = u32::MAX;
        create_row.PrefixOrigin = 1; // IpPrefixOriginManual
        create_row.SuffixOrigin = 1; // IpSuffixOriginManual

        let status = CreateUnicastIpAddressEntry(&create_row);
        if status != NO_ERROR {
            log::error!("CreateUnicastIpAddressEntry failed: {status}");
            return Err(io::Error::from_raw_os_error(status as i32));
        }

        Ok(())
    }
}

pub fn set_default_route(luid: u64, gateway: Ipv4Addr) -> io::Result<()> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        CreateIpForwardEntry2, DeleteIpForwardEntry2, MIB_IPFORWARD_ROW2,
    };
    use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
    use windows_sys::Win32::Networking::WinSock::AF_INET;

    unsafe {
        let mut row: MIB_IPFORWARD_ROW2 = std::mem::zeroed();
        row.InterfaceLuid = NET_LUID_LH { Value: luid };
        row.DestinationPrefix.Prefix.si_family = AF_INET;
        row.DestinationPrefix.Prefix.Ipv4.sin_family = AF_INET;
        row.DestinationPrefix.PrefixLength = 0;
        row.NextHop.si_family = AF_INET;
        row.NextHop.Ipv4.sin_family = AF_INET;
        row.NextHop.Ipv4.sin_addr.S_un.S_addr = u32::from_ne_bytes(gateway.octets());
        row.Metric = 0;
        row.Protocol = 3; // MIB_IPPROTO_NETMGMT
        row.ValidLifetime = u32::MAX;
        row.PreferredLifetime = u32::MAX;

        let del_status = DeleteIpForwardEntry2(&row);
        if del_status != NO_ERROR && del_status != ERROR_NOT_FOUND {
            log::warn!("DeleteIpForwardEntry2 failed: {del_status}");
        }

        let status = CreateIpForwardEntry2(&row);
        if status != NO_ERROR {
            log::error!("CreateIpForwardEntry2 failed: {status}");
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        Ok(())
    }
}

fn set_interface_metric(luid: u64, metric: u32, ipv6: bool) -> io::Result<()> {
    use windows_sys::Win32::NetworkManagement::IpHelper::{
        GetIpInterfaceEntry, MIB_IPINTERFACE_ROW,
    };
    use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
    use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6};

    let luid = NET_LUID_LH { Value: luid };

    let family = if ipv6 { AF_INET6 } else { AF_INET };
    let family_name = if ipv6 { "ipv6" } else { "ipv4" };

    let mut row = MIB_IPINTERFACE_ROW {
        InterfaceLuid: luid,
        Family: family,
        ..Default::default()
    };

    // SAFETY: `row` is initialized and has luid set
    let status = unsafe { GetIpInterfaceEntry(&mut row) };
    if ipv6 && status == ERROR_NOT_FOUND {
        // IPv6 has no IP interface row, e.g. disabled on the host. The metric is
        // a non-essential routing preference, so skip it rather than failing the
        // whole tunnel setup. IPv4 must always be present, so it still errors.
        log::warn!("no IP interface row, skipping metric family={family_name}");
        return Ok(());
    }
    if status != NO_ERROR {
        log::error!("GetIpInterfaceEntry failed with error: {status} family={family_name}");
        return Err(io::Error::from_raw_os_error(status as i32));
    }

    // `SitePrefixLength` must be zeroed and not modified
    row.SitePrefixLength = 0;
    row.Metric = metric;
    row.UseAutomaticMetric = false;

    // SAFETY: `row` is initialized and has luid set
    let status = unsafe { SetIpInterfaceEntry(&mut row) };
    if status != NO_ERROR {
        log::error!("SetIpInterfaceEntry failed with error: {status} family={family_name}");
        return Err(io::Error::from_raw_os_error(status as i32));
    }

    Ok(())
}
