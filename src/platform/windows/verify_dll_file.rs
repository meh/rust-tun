use std::ffi::OsStr;
use std::iter::once;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use windows_sys::core::PCWSTR;
use windows_sys::Win32::Foundation::{
    FreeLibrary, GetLastError, CRYPT_E_SECURITY_SETTINGS, ERROR_SUCCESS, TRUST_E_EXPLICIT_DISTRUST,
    TRUST_E_NOSIGNATURE, TRUST_E_PROVIDER_UNKNOWN, TRUST_E_SUBJECT_FORM_UNKNOWN,
    TRUST_E_SUBJECT_NOT_TRUSTED,
};
use windows_sys::Win32::Security::Cryptography::{
    CertFindCertificateInStore, CertFreeCertificateContext, CertGetNameStringW, CryptMsgGetParam,
    CryptQueryObject, CERT_FIND_SUBJECT_CERT, CERT_INFO, CERT_NAME_SIMPLE_DISPLAY_TYPE,
    CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED, CERT_QUERY_FORMAT_FLAG_BINARY,
    CERT_QUERY_OBJECT_FILE, CMSG_SIGNER_INFO, CMSG_SIGNER_INFO_PARAM,
};
use windows_sys::Win32::Security::WinTrust::{
    WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_DATA_0,
    WINTRUST_FILE_INFO, WTD_CHOICE_FILE, WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE,
    WTD_STATEACTION_VERIFY, WTD_UI_NONE,
};
use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, LoadLibraryW};

/// Verify the embedded signature of a PE file. ref:
/// https://learn.microsoft.com/en-us/windows/win32/seccrypto/example-c-program--verifying-the-signature-of-a-pe-file
pub fn verify_embedded_signature<P>(source_file: P) -> std::io::Result<()>
where
    P: Into<PathBuf>,
{
    let pwsz_source_file = OsStr::new(&source_file.into())
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<u16>>();

    let mut file_data = WINTRUST_FILE_INFO {
        cbStruct: size_of::<WINTRUST_FILE_INFO>() as _,
        pcwszFilePath: pwsz_source_file.as_ptr(),
        hFile: 0 as _,
        pgKnownSubject: null_mut(),
    };

    let win_trust_data_0 = WINTRUST_DATA_0 {
        pFile: &mut file_data,
    };

    let mut win_trust_data = WINTRUST_DATA {
        cbStruct: size_of::<WINTRUST_DATA>() as _,
        pPolicyCallbackData: null_mut(),
        pSIPClientData: null_mut(),
        dwUIChoice: WTD_UI_NONE,
        fdwRevocationChecks: WTD_REVOKE_NONE,
        dwUnionChoice: WTD_CHOICE_FILE,
        Anonymous: win_trust_data_0,
        dwStateAction: WTD_STATEACTION_VERIFY,
        hWVTStateData: 0 as _,
        pwszURLReference: null_mut(),
        dwProvFlags: 0 as _,
        dwUIContext: 0,
        pSignatureSettings: null_mut(),
    };

    let wvt_policy_guid = WINTRUST_ACTION_GENERIC_VERIFY_V2;

    let l_status = unsafe {
        WinVerifyTrust(
            0 as _,
            &wvt_policy_guid as *const _ as *mut _,
            &mut win_trust_data as *mut _ as *mut _,
        )
    };

    const _ERROR_SUCCESS: i32 = ERROR_SUCCESS as _;

    let res = match l_status {
        _ERROR_SUCCESS => {
            Ok(())
        }
        TRUST_E_NOSIGNATURE => {
            let dw_last_error = unsafe { GetLastError() } as i32;
            let err = if dw_last_error == TRUST_E_NOSIGNATURE
                || dw_last_error == TRUST_E_SUBJECT_FORM_UNKNOWN
                || dw_last_error == TRUST_E_PROVIDER_UNKNOWN
            {
                "The file is not signed."
            } else {
                "An unknown error occurred trying to verify the signature of the file."
            };
            Err(std::io::Error::new(std::io::ErrorKind::Other, err))
        }
        TRUST_E_EXPLICIT_DISTRUST => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "The signature is present, but specifically disallowed.",
        )),
        TRUST_E_SUBJECT_NOT_TRUSTED => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "The signature is present, but not trusted.",
        )),
        CRYPT_E_SECURITY_SETTINGS => {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "The hash representing the subject or the publisher wasn't explicitly trusted by the admin and admin policy has disabled user trust. No signature, publisher or timestamp errors.",
            ))
        }
        _ => {
            Err(std::io::Error::from_raw_os_error(l_status))
        }
    };

    // Any hWVTStateData must be released by a call with close.
    win_trust_data.dwStateAction = WTD_STATEACTION_CLOSE;
    let _l_status = unsafe {
        WinVerifyTrust(
            0 as _,
            &wvt_policy_guid as *const _ as *mut _,
            &mut win_trust_data as *mut _ as *mut _,
        )
    };

    res
}

/// Get the signer name of a signed PE file. ref:
/// https://gist.github.com/dougpuob/1cb6c2f16c95d1e7f324d23e76c80f8e
pub fn get_signer_name<P>(source_file: P) -> std::io::Result<String>
where
    P: Into<PathBuf>,
{
    let pwsz_source_file = OsStr::new(&source_file.into())
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<u16>>();

    let mut dw_encoding = 0;
    let mut dw_content_type = 0;
    let mut dw_format_type = 0;
    let mut h_store = 0 as _;
    let mut h_msg = 0 as _;
    let mut dw_signer_info = 0;

    let res = unsafe {
        CryptQueryObject(
            CERT_QUERY_OBJECT_FILE,
            pwsz_source_file.as_ptr() as _,
            CERT_QUERY_CONTENT_FLAG_PKCS7_SIGNED_EMBED,
            CERT_QUERY_FORMAT_FLAG_BINARY,
            0,
            &mut dw_encoding,
            &mut dw_content_type,
            &mut dw_format_type,
            &mut h_store,
            &mut h_msg,
            null_mut(),
        )
    };
    if res == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let res = unsafe {
        CryptMsgGetParam(
            h_msg,
            CMSG_SIGNER_INFO_PARAM,
            0,
            null_mut(),
            &mut dw_signer_info,
        )
    };
    if res == 0 {
        return Err(std::io::Error::last_os_error());
    }
    if dw_signer_info < size_of::<CMSG_SIGNER_INFO>() as u32 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid signer info size.",
        ));
    }

    let mut p_signer_info = vec![0u8; dw_signer_info as usize];
    let res = unsafe {
        CryptMsgGetParam(
            h_msg,
            CMSG_SIGNER_INFO_PARAM,
            0,
            p_signer_info.as_mut_ptr() as _,
            &mut dw_signer_info,
        )
    };
    if res == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let p_signer_info = p_signer_info.as_ptr() as *const CMSG_SIGNER_INFO;

    let mut p_cert_info = vec![0u8; size_of::<CERT_INFO>()];
    let p_cert_info = p_cert_info.as_mut_ptr() as *mut CERT_INFO;
    unsafe { p_cert_info.as_mut().unwrap().Issuer = p_signer_info.as_ref().unwrap().Issuer };
    unsafe {
        p_cert_info.as_mut().unwrap().SerialNumber = p_signer_info.as_ref().unwrap().SerialNumber
    };

    let p_cert_context = unsafe {
        CertFindCertificateInStore(
            h_store,
            dw_encoding,
            0,
            CERT_FIND_SUBJECT_CERT,
            p_cert_info as _,
            null_mut(),
        )
    };
    if p_cert_context.is_null() {
        return Err(std::io::Error::last_os_error());
    }

    let dw_data = unsafe {
        CertGetNameStringW(
            p_cert_context,
            CERT_NAME_SIMPLE_DISPLAY_TYPE,
            0,
            null_mut(),
            null_mut(),
            0,
        )
    };
    if dw_data == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let mut sz_name = vec![0u16; dw_data as usize];
    let res = unsafe {
        CertGetNameStringW(
            p_cert_context,
            CERT_NAME_SIMPLE_DISPLAY_TYPE,
            0,
            null_mut(),
            sz_name.as_mut_ptr(),
            dw_data,
        )
    };
    if res == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let res = String::from_utf16(&sz_name[..sz_name.len() - 1]).map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-16 string.")
    })?;

    unsafe { CertFreeCertificateContext(p_cert_context) };

    Ok(res)
}

pub(crate) unsafe fn get_dll_absolute_path<P>(path: P) -> std::io::Result<std::path::PathBuf>
where
    P: AsRef<::std::ffi::OsStr>,
{
    let path = path.as_ref();
    let wide_filename: Vec<u16> = path.encode_wide().chain(Some(0)).collect();
    // FIXME: `LoadLibraryW` calling will execute the DLL's DllMain function,
    //        which is not desirable, but we haven't a better way to avoid it yet.
    let lib = unsafe { LoadLibraryW(wide_filename.as_ptr() as PCWSTR) };
    let mut buf = [0u16; 1024];
    let len = GetModuleFileNameW(lib, &mut buf as *mut _ as *mut _, buf.len() as _) as usize;
    FreeLibrary(lib);
    if len == 0 {
        return Err(std::io::Error::last_os_error());
    }
    use std::os::windows::ffi::OsStringExt;
    let path2 = std::ffi::OsString::from_wide(&buf[..len])
        .into_string()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string_lossy()))?;

    Ok(std::path::PathBuf::from(path2))
}

#[cfg(test)]
fn get_wintun_bin_pattern_path() -> std::io::Result<std::path::PathBuf> {
    let dll_path = if cfg!(target_arch = "x86") {
        "wintun/bin/x86/wintun.dll"
    } else if cfg!(target_arch = "x86_64") {
        "wintun/bin/amd64/wintun.dll"
    } else if cfg!(target_arch = "arm") {
        "wintun/bin/arm/wintun.dll"
    } else if cfg!(target_arch = "aarch64") {
        "wintun/bin/arm64/wintun.dll"
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported architecture",
        ));
    };
    Ok(dll_path.into())
}

#[cfg(test)]
fn get_crate_dir(crate_name: &str) -> Result<std::path::PathBuf, crate::BoxError> {
    let output = std::process::Command::new("cargo")
        .arg("metadata")
        .arg("--format-version=1")
        .output()?;

    let metadata = serde_json::from_slice::<serde_json::Value>(&output.stdout)?;
    let packages = metadata["packages"].as_array().ok_or("packages")?;

    let mut crate_dir = None;

    for package in packages {
        let name = package["name"].as_str().ok_or("name")?;
        if name == crate_name {
            let path = package["manifest_path"].as_str().ok_or("manifest_path")?;
            let path = std::path::PathBuf::from(path);
            crate_dir = Some(path.parent().ok_or("parent")?.to_path_buf());
            break;
        }
    }
    Ok(crate_dir.ok_or("crate_dir")?)
}

#[test]
fn tests() {
    // The wintun crate's root directory
    let crate_dir = get_crate_dir("wintun").unwrap();

    // The path to the DLL file, relative to the crate root, depending on the target architecture
    let dll_path = get_wintun_bin_pattern_path().unwrap();
    let path = crate_dir.join(dll_path);

    // let path = get_wintun_bin_pattern_path().unwrap();
    verify_embedded_signature(&path).unwrap();
    let signer = get_signer_name(&path).unwrap();
    assert_eq!(signer, super::WINTUN_PROVIDER);
}
