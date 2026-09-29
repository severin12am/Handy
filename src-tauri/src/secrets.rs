//! Post-processing API key resolution.
//!
//! A key typed into settings wins. When that field is empty, Handy reads
//! `HANDY_<PROVIDER>_API_KEY` (for example `HANDY_OPENAI_API_KEY`) and then
//! `HANDY_POST_PROCESS_API_KEY` (cjpais/Handy#483).
//!
//! On Windows a saved key is also copied into Credential Manager so it is
//! available to the process even if the JSON store is blanked later. The JSON
//! store remains the source the settings screen edits; Credential Manager is
//! a backup read when the stored value is empty.

use crate::settings::AppSettings;

pub fn resolve_api_key(provider_id: &str, stored: &str) -> String {
    if !stored.trim().is_empty() {
        return stored.to_string();
    }
    #[cfg(target_os = "windows")]
    if let Some(saved) = read_credential(&credential_target(provider_id)) {
        if !saved.trim().is_empty() {
            return saved;
        }
    }
    let specific = format!(
        "HANDY_{}_API_KEY",
        provider_id.to_ascii_uppercase().replace('-', "_")
    );
    if let Ok(value) = std::env::var(&specific) {
        if !value.trim().is_empty() {
            return value;
        }
    }
    std::env::var("HANDY_POST_PROCESS_API_KEY").unwrap_or_default()
}

/// Copy non-empty keys into Credential Manager. The JSON copy is left intact
/// so the settings screen keeps showing that a key is set.
pub fn backup_api_keys(settings: &AppSettings) {
    #[cfg(target_os = "windows")]
    {
        for (provider, key) in settings.post_process_api_keys.iter() {
            if key.trim().is_empty() {
                continue;
            }
            let _ = write_credential(&credential_target(provider), key);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = settings;
    }
}

#[cfg(target_os = "windows")]
fn credential_target(provider_id: &str) -> String {
    format!("Handy/post-process/{provider_id}")
}

#[cfg(target_os = "windows")]
fn write_credential(target: &str, secret: &str) -> bool {
    use windows::Win32::Security::Credentials::{
        CredWriteW, CREDENTIALW, CRED_FLAGS, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };
    use windows::core::PWSTR;

    let mut target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let mut blob = secret.as_bytes().to_vec();
    let credential = CREDENTIALW {
        Flags: CRED_FLAGS(0),
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(target_wide.as_mut_ptr()),
        Comment: PWSTR::null(),
        LastWritten: Default::default(),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0,
        Attributes: std::ptr::null_mut(),
        TargetAlias: PWSTR::null(),
        UserName: PWSTR::null(),
    };
    unsafe { CredWriteW(&credential, 0).is_ok() }
}

#[cfg(target_os = "windows")]
fn read_credential(target: &str) -> Option<String> {
    use windows::Win32::Security::Credentials::{CredFree, CredReadW, CRED_TYPE_GENERIC};
    use windows::core::PCWSTR;

    let target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let mut credential = std::ptr::null_mut();
    let ok = unsafe {
        CredReadW(
            PCWSTR(target_wide.as_ptr()),
            CRED_TYPE_GENERIC,
            None,
            &mut credential,
        )
        .is_ok()
    };
    if !ok || credential.is_null() {
        return None;
    }
    let text = unsafe {
        let cred = &*credential;
        let bytes = std::slice::from_raw_parts(
            cred.CredentialBlob,
            cred.CredentialBlobSize as usize,
        );
        let text = String::from_utf8_lossy(bytes).into_owned();
        CredFree(credential as *const _);
        text
    };
    Some(text)
}
