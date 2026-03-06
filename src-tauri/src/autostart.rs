#![cfg(windows)]

use std::env;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

const RUN_KEY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_NAME: &str = "AutoDarkMode";

/// Writes or removes the app path in HKCU\...\Run so Windows starts the app at login.
/// Path is quoted if it contains spaces so the Run key works correctly.
pub fn set_autostart(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let exe = env::current_exe()?.into_os_string();
    let path = exe.to_string_lossy();
    let value = if path.contains(' ') {
        format!("\"{}\"", path)
    } else {
        path.to_string()
    };
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags(RUN_KEY_PATH, KEY_READ | KEY_WRITE)?;
    if enabled {
        key.set_value(APP_NAME, &value)?;
    } else {
        let _ = key.delete_value(APP_NAME);
    }
    Ok(())
}
