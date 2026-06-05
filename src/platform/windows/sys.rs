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
use std::os::windows::io::RawHandle;
use std::ptr::NonNull;
use std::sync::Mutex;
use std::time::Duration;

use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, NO_ERROR};
use windows_sys::Win32::NetworkManagement::IpHelper::{
    CancelMibChangeNotify2, GetIpInterfaceEntry, InitializeUnicastIpAddressEntry,
    MIB_IPINTERFACE_ROW, MibAddInstance, NotifyIpInterfaceChange, SetIpInterfaceEntry,
};
use windows_sys::Win32::NetworkManagement::Ndis::NET_LUID_LH;
use windows_sys::Win32::Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC};

use crate::{Error, Result};

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

pub fn set_interface_metric(luid: u64, metric: u32, ipv6: bool) -> io::Result<()> {
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

fn ip_interface_entry_exists(luid: u64, ipv6: bool) -> io::Result<bool> {
    let luid = NET_LUID_LH { Value: luid };
    let family = if ipv6 { AF_INET6 } else { AF_INET };

    let mut row = MIB_IPINTERFACE_ROW {
        InterfaceLuid: luid,
        Family: family,
        ..Default::default()
    };

    // SAFETY: `row` is initialized and has luid set
    match unsafe { GetIpInterfaceEntry(&mut row) } {
        NO_ERROR => Ok(true),
        ERROR_NOT_FOUND => Ok(false),
        other => {
            log::error!("GetIpInterfaceEntry failed with error: {other}");
            Err(io::Error::from_raw_os_error(other as i32))
        }
    }
}

/// Waits until the specified IP interfaces have appeared for a given network device.
/// This fails if the interfaces have not appeared after the specified `timeout`.
pub fn wait_for_interfaces(luid: u64, ipv4: bool, ipv6: bool, timeout: Duration) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    match start_wait_for_interfaces(luid, ipv4, ipv6, tx).map_err(Error::Io)? {
        StartNotifyResult::AlreadyExist => Ok(()),
        StartNotifyResult::Waiting(_handle) => rx
            .recv_timeout(timeout)
            .map_err(|_| Error::InterfaceTimeout),
    }
}

enum StartNotifyResult {
    AlreadyExist,
    Waiting(IpNotifierHandle),
}

/// Begins to wait until the specified IP interfaces have attached to a given network interface.
///
/// `StartNotifyResult::AlreadyExist` is returned if requested interfaces already exist.
///
/// Otherwise, on success, `on_found` is called when all requested interfaces have been added.
/// The wait is cancelled if the returned handle is dropped.
fn start_wait_for_interfaces(
    luid: u64,
    ipv4: bool,
    ipv6: bool,
    on_found: std::sync::mpsc::SyncSender<()>,
) -> io::Result<StartNotifyResult> {
    let mut found_ipv4 = !ipv4;
    let mut found_ipv6 = !ipv6;

    let mut on_found = Some(on_found);

    let handle = notify_ip_interface_change(move |row, notification_type| {
        if found_ipv4 && found_ipv6 {
            return;
        }
        if notification_type != MibAddInstance {
            return;
        }
        // SAFETY: This is always valid as a `u64`.
        if unsafe { row.InterfaceLuid.Value } != luid {
            return;
        }
        match row.Family {
            AF_INET => found_ipv4 = true,
            AF_INET6 => found_ipv6 = true,
            _ => (),
        }
        if found_ipv4
            && found_ipv6
            && let Some(on_found) = on_found.take()
        {
            let _ = on_found.send(());
        }
    })?;

    // Make sure the interfaces were not already up
    if (!ipv4 || ip_interface_entry_exists(luid, false)?)
        && (!ipv6 || ip_interface_entry_exists(luid, true)?)
    {
        return Ok(StartNotifyResult::AlreadyExist);
    }

    Ok(StartNotifyResult::Waiting(handle))
}

type InnerCallback = Box<Mutex<dyn FnMut(&MIB_IPINTERFACE_ROW, i32) + Send + 'static>>;

/// Context for [`notify_ip_interface_change`]. When it is dropped, the callback is unregistered.
pub struct IpNotifierHandle {
    callback: Option<NonNull<InnerCallback>>,
    handle: RawHandle,
}

impl Drop for IpNotifierHandle {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // SAFETY: `self.handle` is a valid notify handle that we own
            unsafe { CancelMibChangeNotify2(self.handle) };
        }

        let callback = self
            .callback
            .take()
            .expect("callback is Some until drop is called");
        let callback = callback.as_ptr();
        // SAFETY:
        // - Callback was constructed in `notify_ip_interface_change` using `Box::into_raw`.
        // - `CancelMibChangeNotify2` ensures that the callback is removed, so we can safely take ownership.
        let _inner_callback: Box<InnerCallback> = unsafe { Box::from_raw(callback) };
    }
}

/// Registers a callback function that is invoked when an interface is added, removed,
/// or changed.
pub fn notify_ip_interface_change<T: FnMut(&MIB_IPINTERFACE_ROW, i32) + Send + 'static>(
    callback: T,
) -> io::Result<IpNotifierHandle> {
    // Box mutex because fat pointer
    let callback = Box::new(Mutex::new(callback)) as Box<Mutex<_>>;
    let callback: Box<InnerCallback> = Box::new(callback);
    let callback = NonNull::new(Box::into_raw(callback)).unwrap();

    let mut context = IpNotifierHandle {
        callback: Some(callback),
        handle: std::ptr::null_mut(),
    };

    let status = unsafe {
        NotifyIpInterfaceChange(
            AF_UNSPEC,
            Some(outer_callback),
            callback.as_ptr().cast(),
            false,
            &raw mut context.handle,
        )
    };
    if status != NO_ERROR {
        return Err(::std::io::Error::from_raw_os_error(status as i32));
    }
    Ok(context)
}

unsafe extern "system" fn outer_callback(
    context: *const std::ffi::c_void,
    row: *const MIB_IPINTERFACE_ROW,
    notify_type: i32,
) {
    // SAFETY: `context` is a valid pointer to an `InnerCallback` constructed in `notify_ip_interface_change`.
    // `outer_callback` is never called after `CancelMibChangeNotify2` has completed, and `CancelMibChangeNotify2`
    // blocks until the function returns if it is currently being called.
    let cb = unsafe { &*context.cast::<InnerCallback>() };
    // SAFETY: `row` is set when type is not `MibInitialNotification`, which we do not use.
    let row = unsafe { &*row };
    cb.lock().expect("NotifyIpInterfaceChange mutex poisoned")(row, notify_type);
}
