// Windows registry theme switch (System + Apps light/dark) + DWM refresh
#![cfg(windows)]

use std::thread;
use std::time::Duration;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
use winreg::RegKey;

use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{InvalidateRect, UpdateWindow};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumChildWindows, EnumWindows, GetClassNameW, PostMessageW, SendMessageTimeoutW, SetWindowPos,
    HWND_TOP, SMTO_ABORTIFHUNG, SMTO_BLOCK, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE,
    SWP_NOSIZE, SWP_NOZORDER, WM_SETTINGCHANGE, WM_SYSCOLORCHANGE, WM_THEMECHANGED,
};

// WM_DWMCOLORIZATIONCOLORCHANGED = 0x0320 (winuser.h)
const WM_DWMCOLORIZATIONCOLORCHANGED: u32 = 0x0320;

const PERSONALIZE_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const DWM_PATH: &str = r"Software\Microsoft\Windows\DWM";
const APPS_LIGHT: &str = "AppsUseLightTheme";
const SYSTEM_LIGHT: &str = "SystemUsesLightTheme";
const DWM_COLORIZATION: &str = "ColorizationColor";
const TIMEOUT_MS: u32 = 5000;
const DWM_REFRESH_SLEEP_MS: u64 = 600;
const TASKBAR_REFRESH_STEP_MS: u64 = 150;
/// Delay before repeating broadcasts so secondary monitor taskbar can catch up
const DELAYED_BROADCAST_MS: u64 = 1500;
const REPEAT_BROADCAST_INTERVAL_MS: u64 = 300;
const REPEAT_BROADCAST_COUNT: u32 = 3;
/// Duration the primary taskbar will briefly flash to the opposite theme during
/// the precondition step of the multi-monitor hack. Kept short so the flicker
/// is barely perceptible, but long enough for Shell_SecondaryTrayWnd's theme
/// cache to be invalidated by a real state transition.
const PRECONDITION_FLASH_MS: u64 = 120;
const SETTING_CHANGE_TOPICS: [&str; 2] = ["ImmersiveColorSet", "WindowsThemeElement"];
const SHELL_TASKBAR_CLASSES: [&str; 2] = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];
const SECONDARY_TASKBAR_CLASS: &str = "Shell_SecondaryTrayWnd";

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

unsafe extern "system" fn collect_secondary_taskbars(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<HWND>);
    if let Some(class_name) = window_class_name(hwnd) {
        if class_name.as_str() == SECONDARY_TASKBAR_CLASS {
            windows.push(hwnd);
        }
    }
    BOOL(1)
}

fn enumerate_secondary_taskbars() -> Vec<HWND> {
    let mut taskbars = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect_secondary_taskbars),
            LPARAM((&mut taskbars as *mut Vec<HWND>) as isize),
        );
    }
    taskbars
}

/// Aggressively "nudge" a secondary taskbar so its window procedure actually
/// processes the subsequent theme message. This mirrors the empirical
/// workaround some users report: right-clicking the secondary taskbar (which
/// wakes it up via non-client hit testing) before the theme switch makes the
/// switch land correctly. We simulate that wake-up purely via the message
/// queue without any visible side effects, and also force a non-client frame
/// recalc so the taskbar reconsiders its theme brushes.
fn nudge_secondary_taskbar(hwnd: HWND) {
    // WM_NCHITTEST + WM_NCMOUSEMOVE: simulate a non-client mouse interaction
    // without a real input event. These are queued via PostMessage so they go
    // into the taskbar's own thread queue and actually execute in its window
    // procedure, draining any stale message state.
    const WM_NCHITTEST: u32 = 0x0084;
    const WM_NCMOUSEMOVE: u32 = 0x00A0;
    const HTCAPTION: isize = 2;
    unsafe {
        let _ = PostMessageW(hwnd, WM_NCHITTEST, WPARAM(0), LPARAM(0));
        let _ = PostMessageW(hwnd, WM_NCMOUSEMOVE, WPARAM(HTCAPTION as usize), LPARAM(0));
        // SWP_FRAMECHANGED forces WM_NCCALCSIZE, which on Win11 taskbars
        // re-queries the current system theme for non-client brushes.
        let _ = SetWindowPos(
            hwnd,
            HWND_TOP,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );
    }
}

fn nudge_all_secondary_taskbars() {
    for hwnd in enumerate_secondary_taskbars() {
        nudge_secondary_taskbar(hwnd);
    }
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

fn write_theme_values(
    key: &winreg::RegKey,
    value: u32,
    switch_system: bool,
    switch_apps: bool,
) -> std::io::Result<()> {
    if switch_apps {
        key.set_value(APPS_LIGHT, &value)?;
    }
    if switch_system {
        key.set_value(SYSTEM_LIGHT, &value)?;
    }
    Ok(())
}

pub fn set_theme(
    light: bool,
    switch_system: bool,
    switch_apps: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let target = if light { 1u32 } else { 0u32 };
    let opposite = if light { 0u32 } else { 1u32 };
    let key = open_personalize(true)?;

    // ------------------------------------------------------------------
    // Multi-monitor Shell_SecondaryTrayWnd theme-cache hack
    // ------------------------------------------------------------------
    // Confirmed Windows 11 bug (also reproducible via the native Settings
    // app, and present in Microsoft's own PowerToys LightSwitch as well as
    // Auto Dark Mode): after a theme switch, secondary monitor taskbars
    // sometimes keep rendering the previous theme until the user toggles
    // the theme a second time. Broadcasting WM_THEMECHANGED /
    // WM_SETTINGCHANGE / DWM refreshes is not reliably enough on its own,
    // because Shell_SecondaryTrayWnd caches a "last applied" theme state
    // that can go out of sync with the registry and then get skipped.
    //
    // The one workaround that works reliably on affected systems is
    // exactly what the user does manually: force the taskbar through a
    // real state transition rather than a repeat of the same value. We do
    // that here by first writing the opposite value and broadcasting, then
    // immediately writing the target value and broadcasting again. The
    // primary taskbar briefly (~PRECONDITION_FLASH_MS) flashes to the
    // opposite theme, but this is barely perceptible and is strictly
    // better than leaving the secondary taskbar stuck in the wrong theme.
    //
    // We skip the precondition when the effective target matches the
    // current state (no real transition needed) so we don't add flicker
    // for no reason.
    let current_apps: u32 = key.get_value(APPS_LIGHT).unwrap_or(1);
    let current_system: u32 = key.get_value(SYSTEM_LIGHT).unwrap_or(1);
    let apps_changes = switch_apps && current_apps != target;
    let system_changes = switch_system && current_system != target;
    let is_real_transition = apps_changes || system_changes;

    if is_real_transition {
        // Precondition: write opposite, then broadcast + nudge the
        // secondary taskbars so their internal theme cache observes a real
        // opposite state, not a no-op repeat.
        write_theme_values(&key, opposite, switch_system, switch_apps)?;
        nudge_all_secondary_taskbars();
        refresh_shell_ui();
        thread::sleep(Duration::from_millis(PRECONDITION_FLASH_MS));
    }

    // Final target state.
    write_theme_values(&key, target, switch_system, switch_apps)?;
    nudge_all_secondary_taskbars();
    refresh_shell_ui();
    thread::sleep(Duration::from_millis(TASKBAR_REFRESH_STEP_MS));
    refresh_dwm_via_colorization();
    refresh_shell_ui();

    // Delayed repeat: secondary monitor taskbar (Shell_SecondaryTrayWnd) often
    // processes theme changes one cycle behind the primary taskbar. Same logic
    // for both manual (Dashboard button) and scheduler (first run / timer).
    // First wave at DELAYED_BROADCAST_MS; second wave later for slow-to-appear
    // secondary taskbars (e.g. right after app start or monitor wake).
    const SECOND_WAVE_DELAY_MS: u64 = 3500;
    std::thread::spawn(|| {
        for wave_delay in [DELAYED_BROADCAST_MS, SECOND_WAVE_DELAY_MS] {
            thread::sleep(Duration::from_millis(wave_delay));
            for _ in 0..REPEAT_BROADCAST_COUNT {
                nudge_all_secondary_taskbars();
                refresh_shell_taskbars();
                thread::sleep(Duration::from_millis(REPEAT_BROADCAST_INTERVAL_MS));
            }
        }
    });
    Ok(())
}
