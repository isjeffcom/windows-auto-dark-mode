// Windows registry theme switch (System + Apps light/dark) + DWM refresh
#![cfg(windows)]

use std::thread;
use std::time::Duration;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_BLOCK, WM_SETTINGCHANGE, WM_THEMECHANGED,
};

// WM_DWMCOLORIZATIONCOLORCHANGED = 0x0320 (winuser.h)
const WM_DWMCOLORIZATIONCOLORCHANGED: u32 = 0x0320;

const PERSONALIZE_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const DWM_PATH: &str = r"Software\Microsoft\Windows\DWM";
const APPS_LIGHT: &str = "AppsUseLightTheme";
const SYSTEM_LIGHT: &str = "SystemUsesLightTheme";
const DWM_COLORIZATION: &str = "ColorizationColor";
const TIMEOUT_MS: u32 = 5000;
const DWM_REFRESH_SLEEP_MS: u64 = 800;
/// Delay before one more full refresh so Explorer surfaces on secondary monitors can catch up.
const DELAYED_FULL_REFRESH_MS: u64 = 1400;
const FOLLOW_UP_BROADCAST_INTERVAL_MS: u64 = 300;
const FOLLOW_UP_BROADCAST_COUNT: u32 = 3;

fn hwnd_broadcast() -> HWND {
    HWND(0xffff as *mut std::ffi::c_void)
}

fn open_personalize(write: bool) -> std::io::Result<winreg::RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if write {
        hkcu.open_subkey_with_flags(PERSONALIZE_PATH, KEY_READ | KEY_WRITE)
    } else {
        hkcu.open_subkey_with_flags(PERSONALIZE_PATH, KEY_READ)
    }
}

fn open_dwm(write: bool) -> std::io::Result<winreg::RegKey> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if write {
        hkcu.open_subkey_with_flags(DWM_PATH, KEY_READ | KEY_WRITE)
    } else {
        hkcu.open_subkey_with_flags(DWM_PATH, KEY_READ)
    }
}

/// Read DWM ColorizationColor (0xAARRGGBB) from registry. Returns None if key/value missing.
fn get_dwm_colorization_color() -> Option<u32> {
    let key = open_dwm(false).ok()?;
    key.get_value(DWM_COLORIZATION).ok()
}

/// Write DWM ColorizationColor to registry only.
fn set_dwm_colorization_color(value: u32) -> std::io::Result<()> {
    let key = open_dwm(true)?;
    key.set_value(DWM_COLORIZATION, &value)?;
    key.flush()?;
    Ok(())
}

/// Broadcast WM_DWMCOLORIZATIONCOLORCHANGED so DWM/taskbar repaint (wParam = color 0xAARRGGBB, lParam = blend).
fn broadcast_dwm_colorization(color: u32, blend: bool) {
    unsafe {
        let _ = SendMessageTimeoutW(
            hwnd_broadcast(),
            WM_DWMCOLORIZATIONCOLORCHANGED,
            WPARAM(color as _),
            LPARAM(if blend { 1 } else { 0 }),
            SMTO_ABORTIFHUNG,
            TIMEOUT_MS,
            None,
        );
    }
}

/// Force DWM/taskbar to refresh by briefly changing ColorizationColor then restoring (like Windows-Auto-Night-Mode).
fn refresh_dwm_via_colorization() {
    let Some(original) = get_dwm_colorization_color() else {
        return;
    };
    // Tweak one digit so DWM sees a change (reference: last hex digit +/- 1)
    let tweaked = original ^ 1u32;
    if let Ok(()) = set_dwm_colorization_color(tweaked) {
        broadcast_dwm_colorization(tweaked, true);
        thread::sleep(Duration::from_millis(DWM_REFRESH_SLEEP_MS));
    }
    let _ = set_dwm_colorization_color(original);
    broadcast_dwm_colorization(original, true);
}

/// Notify Windows to refresh taskbar and system UI after registry change.
/// Uses SMTO_BLOCK so we wait for processing (helps secondary monitor taskbar).
fn broadcast_theme_change() {
    let flags = SMTO_BLOCK | SMTO_ABORTIFHUNG;
    unsafe {
        let immersive: Vec<u16> = "ImmersiveColorSet\0".encode_utf16().collect();
        let _ = SendMessageTimeoutW(
            hwnd_broadcast(),
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(immersive.as_ptr() as isize),
            flags,
            TIMEOUT_MS,
            None,
        );
        // Some Explorer surfaces only repaint after a generic settings change broadcast.
        let _ = SendMessageTimeoutW(
            hwnd_broadcast(),
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(0),
            flags,
            TIMEOUT_MS,
            None,
        );
        let _ = SendMessageTimeoutW(
            hwnd_broadcast(),
            WM_THEMECHANGED,
            WPARAM(0),
            LPARAM(0),
            flags,
            TIMEOUT_MS,
            None,
        );
    }
}

pub fn get_theme_state() -> Result<super::ThemeState, Box<dyn std::error::Error>> {
    let key = open_personalize(false)?;
    let apps: u32 = key.get_value(APPS_LIGHT).unwrap_or(1);
    let system: u32 = key.get_value(SYSTEM_LIGHT).unwrap_or(1);
    Ok(super::ThemeState {
        is_light: apps == 1 && system == 1,
        apps_light: apps == 1,
        system_light: system == 1,
    })
}

pub fn set_theme(
    light: bool,
    switch_system: bool,
    switch_apps: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let value = if light { 1u32 } else { 0u32 };
    let key = open_personalize(true)?;
    if switch_apps {
        key.set_value(APPS_LIGHT, &value)?;
    }
    if switch_system {
        key.set_value(SYSTEM_LIGHT, &value)?;
    }
    // Explorer may read these values during WM_SETTINGCHANGE handling, so flush first.
    key.flush()?;
    broadcast_theme_change();
    refresh_dwm_via_colorization();
    broadcast_theme_change();
    // Delayed retry: secondary monitor taskbar can still lag one update behind, so give
    // DWM and Explorer one more nudge after the registry values are fully visible.
    std::thread::spawn(|| {
        thread::sleep(Duration::from_millis(DELAYED_FULL_REFRESH_MS));
        refresh_dwm_via_colorization();
        for _ in 0..FOLLOW_UP_BROADCAST_COUNT {
            broadcast_theme_change();
            thread::sleep(Duration::from_millis(FOLLOW_UP_BROADCAST_INTERVAL_MS));
        }
    });
    Ok(())
}
