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
        Ok(ThemeState { is_light: false, apps_light: false, system_light: false })
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
    state.config.lock().map_err(|e| e.to_string()).map(|c| c.clone())
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
fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
fn get_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

fn tray_menu_labels(lang: &str) -> (String, String) {
    match lang {
        "zh" => ("显示".to_string(), "退出".to_string()),
        _ => ("Show".to_string(), "Quit".to_string()),
    }
}

// Static storage for tray menu items so update_tray_labels can update text.
#[cfg(target_os = "windows")]
static TRAY_ITEMS: std::sync::OnceLock<(
    tauri::menu::MenuItem<tauri::Wry>,
    tauri::menu::MenuItem<tauri::Wry>,
)> = std::sync::OnceLock::new();

#[tauri::command]
fn update_tray_labels(lang: String) {
    #[cfg(target_os = "windows")]
    {
        if let Some((show, quit)) = TRAY_ITEMS.get() {
            let (show_text, quit_text) = tray_menu_labels(&lang);
            let _ = show.set_text(&show_text);
            let _ = quit.set_text(&quit_text);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = lang;
    }
}

pub fn run() {
    let config = AppConfig::load().unwrap_or_else(|_| AppConfig::default());
    let scheduler = Scheduler::start(config.clone());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
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
            // Apply Windows 11 Acrylic effect.
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                let window = app.get_webview_window("main").unwrap();
                let _ = window_vibrancy::apply_acrylic(&window, Some((10, 10, 10, 248)));
            }

            // Native tray icon via Tauri 2 built-in API.
            #[cfg(target_os = "windows")]
            {
                use tauri::Manager;
                use tauri::menu::MenuItem;
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};

                let lang: String = app
                    .try_state::<AppState>()
                    .and_then(|s| s.config.lock().ok().map(|c| c.language.clone()))
                    .unwrap_or_else(|| "en".to_string());
                let (show_text, quit_text) = tray_menu_labels(&lang);

                let show_i = MenuItem::with_id(app, "show", show_text.as_str(), true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", quit_text.as_str(), true, None::<&str>)?;

                // Store for later label updates.
                let _ = TRAY_ITEMS.set((show_i.clone(), quit_i.clone()));

                let menu = tauri::menu::Menu::with_items(app, &[&show_i, &quit_i])?;

                let icon = app
                    .default_window_icon()
                    .ok_or("no app icon configured")?
                    .clone();

                let tray = TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("Auto Dark Mode")
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => std::process::exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::DoubleClick { .. } = event {
                            if let Some(w) = tray.app_handle().get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.unminimize();
                                let _ = w.set_focus();
                            }
                        }
                    })
                    .build(app)?;

                // Leak so the tray icon lives for the app lifetime.
                std::mem::forget(tray);
            }

            // Ctrl+C in terminal also exits cleanly.
            let handle = app.handle().clone();
            let _ = ctrlc::set_handler(move || {
                std::process::exit(0);
            });
            let _ = handle;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
