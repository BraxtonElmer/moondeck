//! API keys and passwords live in Windows Credential Manager, never in config files.
//! The UI can set or clear them and ask whether one exists, but never read one back.

use windows::{
    core::{PCWSTR, PWSTR},
    Win32::Security::Credentials::{
        CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    },
};

/// Secrets the UI is allowed to manage.
pub const NAMES: &[&str] = &["ai", "gmail", "outlook", "hoyolab"];

fn target(name: &str) -> Vec<u16> {
    format!("moondeck:{name}").encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn set(name: &str, value: &str) -> Result<(), String> {
    let mut target = target(name);
    let blob = value.as_bytes();
    let cred = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target.as_mut_ptr()),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_ptr() as *mut u8,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };
    unsafe { CredWriteW(&cred, 0) }.map_err(|e| e.to_string())
}

pub fn get(name: &str) -> Option<String> {
    let target = target(name);
    let mut cred: *mut CREDENTIALW = std::ptr::null_mut();
    unsafe {
        CredReadW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, None, &mut cred).ok()?;
        let c = &*cred;
        let bytes = std::slice::from_raw_parts(c.CredentialBlob, c.CredentialBlobSize as usize).to_vec();
        CredFree(cred as *const _);
        String::from_utf8(bytes).ok()
    }
}

pub fn has(name: &str) -> bool {
    get(name).is_some_and(|v| !v.is_empty())
}

pub fn clear(name: &str) -> Result<(), String> {
    let target = target(name);
    match unsafe { CredDeleteW(PCWSTR(target.as_ptr()), CRED_TYPE_GENERIC, None) } {
        Ok(()) => Ok(()),
        Err(e) if !has(name) => {
            let _ = e;
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore] // touches the real Credential Manager
    fn round_trip() {
        super::set("test", "s3cret").unwrap();
        assert_eq!(super::get("test").as_deref(), Some("s3cret"));
        super::clear("test").unwrap();
        assert!(!super::has("test"));
    }
}
