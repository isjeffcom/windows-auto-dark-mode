// Windows registry theme switch (System + Apps light/dark) + DWM refresh
#![cfg(windows)]

use std::thread;
use std::time::Duration;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetClassNameW, InvalidateRect, SendMessageTimeoutW,
    UpdateWindow, SMTO_ABORTIFHUNG, SMTO_BLOCK, WM_SETTINGCHANGE, WM_SYSCOLORCHANGE,
    WM_THEMECHANGED,
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
/// Delay before repeating broadcasts so secondary monitor taskbar can catch up
const DELAYED_BROADCAST_MS: u64 = 1600;
const REPEAT_BROADCAST_INTERVAL_MS: u64 = 250;
const REPEAT_BROADCAST_COUNT: u32 = 3;
const TASKBAR_REFRESH_STEP_MS: u64 = 120;
const SETTING_CHANGE_TOPICS: [&str; 2] = ["ImmersiveColorSet", "WindowsThemeElement"];
const SHELL_TASKBAR_CLASSES: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];

fn hwnd_broadcast() -> HWND {
    HWND(0xffff as *mut std::ffi::c_void)
}

fn send_message(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) {
    unsafe {
        let _ = SendMessageTimeoutW(
            hwnd,
            message,
            wparam,
            lparam,
            SMTO_BLOCK | SMTO_ABORTIFHUNG,
            TIMEOUT_MS,
            None,
        );
    }
}

fn send_setting_change(hwnd: HWND, topic: Option<&str>) {
    let topic_utf16 = topic.map(|value| {
        let mut encoded: Vec<u16> = value.encode_utf16().collect();
        encoded.push(0);
        encoded
    });
    let lparam = topic_utf16
        .as_ref()
        .map(|value| LPARAM(value.as_ptr() as isize))
        .unwrap_or(LPARAM(0));
    send_message(hwnd, WM_SETTINGCHANGE, WPARAM(0), lparam);
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
    Ok(())
}

/// Broadcast WM_DWMCOLORIZATIONCOLORCHANGED so DWM/taskbar repaint (wParam = color 0xAARRGGBB, lParam = blend).
fn broadcast_dwm_colorization(color: u32, blend: bool) {
    send_message(
        hwnd_broadcast(),
        WM_DWMCOLORIZATIONCOLORCHANGED,
        WPARAM(color as _),
        LPARAM(if blend { 1 } else { 0 }),
    );
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
    for topic in SETTING_CHANGE_TOPICS {
        send_setting_change(hwnd_broadcast(), Some(topic));
    }
    // Some Explorer surfaces only react to a generic settings-change broadcast.
    send_setting_change(hwnd_broadcast(), None);
    send_message(hwnd_broadcast(), WM_THEMECHANGED, WPARAM(0), LPARAM(0));
    send_message(hwnd_broadcast(), WM_SYSCOLORCHANGE, WPARAM(0), LPARAM(0));
}

fn window_class_name(hwnd: HWND) -> Option<String> {
    let mut class_name = [0u16; 256];
    let length = unsafe { GetClassNameW(hwnd, &mut class_name) };
    if length == 0 {
        return None;
    }
    Some(String::from_utf16_lossy(&class_name[..length as usize]))
}

unsafe extern "system" fn collect_shell_taskbars(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<HWND>);
    if let Some(class_name) = window_class_name(hwnd) {
        if SHELL_TASKBAR_CLASSES.contains(&class_name.as_str()) {
            windows.push(hwnd);
        }
    }
    BOOL(1)
}

unsafe extern "system" fn collect_child_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<HWND>);
    windows.push(hwnd);
    BOOL(1)
}

fn refresh_window(hwnd: HWND) {
    for topic in SETTING_CHANGE_TOPICS {
        send_setting_change(hwnd, Some(topic));
    }
    send_setting_change(hwnd, None);
    send_message(hwnd, WM_THEMECHANGED, WPARAM(0), LPARAM(0));
    send_message(hwnd, WM_SYSCOLORCHANGE, WPARAM(0), LPARAM(0));
    unsafe {
        let _ = InvalidateRect(hwnd, None, BOOL(1));
        let _ = UpdateWindow(hwnd);
    }
}

fn refresh_shell_taskbars() {
    let mut taskbars = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_shell_taskbars),
            LPARAM((&mut taskbars as *mut Vec<HWND>) as isize),
        );
    }

    for taskbar in taskbars {
        refresh_window(taskbar);
        let mut child_windows = Vec::new();
        unsafe {
            let _ = EnumChildWindows(
                taskbar,
                Some(collect_child_windows),
                LPARAM((&mut child_windows as *mut Vec<HWND>) as isize),
            );
        }
        for child in child_windows {
            refresh_window(child);
        }
    }
}

fn refresh_shell_ui() {
    broadcast_theme_change();
    refresh_shell_taskbars();
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
    refresh_shell_ui();
    thread::sleep(Duration::from_millis(TASKBAR_REFRESH_STEP_MS));
    refresh_dwm_via_colorization();
    refresh_shell_ui();
    // Delayed repeat: secondary monitor taskbar often updates one step behind; give it
    // another round of broadcasts so it catches up.
    std::thread::spawn(|| {
        thread::sleep(Duration::from_millis(DELAYED_BROADCAST_MS));
        for _ in 0..REPEAT_BROADCAST_COUNT {
            refresh_shell_ui();
            thread::sleep(Duration::from_millis(REPEAT_BROADCAST_INTERVAL_MS));
        }
    });
    Ok(())
}
