#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod scheduler;
#[cfg(windows)]
mod autostart;
#[cfg(windows)]
mod theme;

use config::AppConfig;
use scheduler::Scheduler;
use std::sync::Mutex;
use tauri::State;

/// Thread-local storage for tray menu items (stored on main thread, accessed via run_on_main_thread).
#[cfg(target_os = "windows")]
use std::cell::RefCell;

#[cfg(target_os = "windows")]
thread_local! {
    static TRAY_ITEMS: RefCell<Option<(tray_icon::menu::MenuItem, tray_icon::menu::MenuItem)>> = RefCell::new(None);
}

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub scheduler: Mutex<Option<Scheduler>>,
}

#[derive(serde::Serialize, Clone)]
pub struct ThemeState {
    pub is_light: bool,
    pub apps_light: bool,
    pub system_light: bool,
}

#[derive(serde::Serialize, Clone)]
pub struct SunTimes {
    pub sunrise: String,
    pub sunset: String,
    pub next_switch: String,
    pub next_switch_is_dark: bool,
}

#[tauri::command]
fn get_theme_state() -> Result<ThemeState, String> {
    #[cfg(windows)]
    {
        theme::get_theme_state().map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        Ok(ThemeState {
            is_light: false,
            apps_light: false,
            system_light: false,
        })
    }
}

#[tauri::command]
fn set_theme(state: State<AppState>, light: bool, is_manual: Option<bool>) -> Result<(), String> {
    #[cfg(windows)]
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?.clone();
        theme::set_theme(light, config.switch_system, config.switch_apps).map_err(|e| e.to_string())?;
        if is_manual.unwrap_or(true) {
            if let Some(until_ts) = scheduler::get_next_boundary_timestamp(&config) {
                config.manual_override_until = Some(until_ts);
                let _ = config.save();
                *state.config.lock().map_err(|e| e.to_string())? = config;
            }
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (state, light, is_manual);
        Ok(())
    }
}

#[tauri::command]
fn get_config(state: State<AppState>) -> Result<AppConfig, String> {
    state
        .config
        .lock()
        .map_err(|e| e.to_string())
        .map(|c| c.clone())
}

#[tauri::command]
fn save_config(state: State<AppState>, config: AppConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    *state.config.lock().map_err(|e| e.to_string())? = config.clone();
    scheduler::restart_scheduler(&state, &config)?;
    #[cfg(windows)]
    {
        let _ = autostart::set_autostart(config.autostart);
    }
    Ok(())
}

#[tauri::command]
fn get_sun_times(state: State<AppState>) -> Result<Option<SunTimes>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    scheduler::compute_sun_times(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_next_switch(state: State<AppState>) -> Result<Option<(String, bool)>, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    scheduler::get_next_switch_time(&config).map_err(|e| e.to_string())
}

#[tauri::command]
fn exit_app(app: tauri::AppHandle) {
    let _ = app.exit(0);
}

/// App version (from tauri.conf.json — single source of truth).
#[tauri::command]
fn get_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

/// Tray menu label strings by language (config.language: "en" | "zh").
fn tray_menu_labels(lang: &str) -> (String, String) {
    match lang {
        "zh" => ("显示".to_string(), "退出".to_string()),
        _ => ("Show".to_string(), "Quit".to_string()),
    }
}

/// Update tray menu item labels (called from frontend when language changes).
#[tauri::command]
fn update_tray_labels(app: tauri::AppHandle, lang: String) {
    #[cfg(target_os = "windows")]
    {
        let (show_text, quit_text) = tray_menu_labels(&lang);
        let _ = app.run_on_main_thread(move || {
            TRAY_ITEMS.with(|cell| {
                if let Some((show, quit)) = cell.borrow().as_ref() {
                    show.set_text(show_text.as_str());
                    quit.set_text(quit_text.as_str());
                }
            });
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, lang);
    }
}

pub fn run() {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let scheduler = Scheduler::start(config.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            config: Mutex::new(config.clone()),
            scheduler: Mutex::new(Some(scheduler)),
        })
        .invoke_handler(tauri::generate_handler![
            get_theme_state,
            set_theme,
            get_config,
            save_config,
            get_sun_times,
            get_next_switch,
            get_version,
            exit_app,
            update_tray_labels,
        ])
        .setup(|app| {
            // Apply Windows 11 Acrylic effect to the main window.
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                // dark-tinted acrylic: rgba(10, 10, 10, 248) — near-opaque, barely any bleed-through
                let _ = window_vibrancy::apply_acrylic(&window, Some((10, 10, 10, 248)));
            }

            // System tray icon (Rust-native so it always shows on Windows).
            #[cfg(target_os = "windows")]
            {
                use std::thread;
                use tauri::Manager;
                use tray_icon::menu::{Menu, MenuEvent, MenuItem};
                use tray_icon::TrayIconBuilder;

                let icon_path = app
                    .path()
                    .resource_dir()
                    .ok()
                    .and_then(|d| {
                        let p = d.join("icons").join("Tray_Icon.ico");
                        if p.exists() { Some(p) } else { None }
                    })
                    .or_else(|| {
                        let p =
                            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icons").join("Tray_Icon.ico");
                        if p.exists() { Some(p) } else { None }
                    });

                let lang: String = app
                    .try_state::<AppState>()
                    .and_then(|s| s.config.lock().ok().map(|c| c.language.clone()))
                    .unwrap_or_else(|| "en".to_string());
                let (show_text, quit_text) = tray_menu_labels(&lang);

                if let Some(path) = icon_path {
                    if let Ok(rgba) = image::open(&path).map(|i| i.into_rgba8()) {
                        let (w, h) = rgba.dimensions();
                        if let Ok(icon) = tray_icon::Icon::from_rgba(rgba.into_raw(), w, h) {
                            let show_i = MenuItem::with_id("show", show_text.as_str(), true, None);
                            let quit_i = MenuItem::with_id("quit", quit_text.as_str(), true, None);

                            // Store clones in thread-local so update_tray_labels can update text later.
                            TRAY_ITEMS.with(|cell| {
                                *cell.borrow_mut() = Some((show_i.clone(), quit_i.clone()));
                            });

                            let menu = Menu::new();
                            let _ = menu.append(&show_i);
                            let _ = menu.append(&quit_i);

                            if let Ok(tray) = TrayIconBuilder::new()
                                .with_icon(icon)
                                .with_tooltip("Auto Dark Mode")
                                .with_menu(Box::new(menu))
                                .build()
                            {
                                let app_handle = app.handle().clone();

                                thread::spawn(move || {
                                    while let Ok(event) = MenuEvent::receiver().recv() {
                                        let id_str = format!("{:?}", event.id);
                                        let id = id_str.trim_matches('"');
                                        match id {
                                            "show" => {
                                                let app = app_handle.clone();
                                                let _ = app_handle.run_on_main_thread(move || {
                                                    if let Some(w) = app.get_webview_window("main") {
                                                        let _ = w.show();
                                                        let _ = w.set_focus();
                                                    }
                                                });
                                            }
                                            "quit" => {
                                                app_handle.exit(0);
                                            }
                                            _ => {}
                                        }
                                    }
                                });

                                // Keep tray alive for app lifetime (TrayIcon is !Send so we leak it).
                                std::mem::forget(tray);
                            }
                        }
                    }
                }
            }

            // When terminal receives Ctrl+C, exit the app too.
            let handle = app.handle().clone();
            let _ = ctrlc::set_handler(move || {
                let _ = handle.exit(0);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
