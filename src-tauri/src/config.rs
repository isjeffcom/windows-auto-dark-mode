use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("AutoDarkMode")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(".")
    }
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub enabled: bool,
    pub mode: ScheduleMode,
    #[serde(default = "default_sunrise")]
    pub sunrise_time: String,
    #[serde(default = "default_sunset")]
    pub sunset_time: String,
    pub location: LocationConfig,
    pub switch_system: bool,
    pub switch_apps: bool,
    pub autostart: bool,
    pub show_tray: bool,
    pub notify_on_switch: bool,
    /// UI language: "en" or "zh" (saved when user changes in Settings; used for tray menu labels).
    #[serde(default = "default_language")]
    pub language: String,
    /// If set, scheduler will not auto-switch until this time (unix timestamp). Set when user manually switches theme.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_override_until: Option<i64>,
}

fn default_language() -> String {
    "en".to_string()
}

fn default_sunrise() -> String {
    "07:00".to_string()
}
fn default_sunset() -> String {
    "19:00".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ScheduleMode {
    /// Fixed time (sunrise_time / sunset_time)
    #[default]
    Fixed,
    /// Sunrise/sunset by location (lat/lon)
    Location,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: ScheduleMode::Fixed,
            sunrise_time: "07:00".to_string(),
            sunset_time: "19:00".to_string(),
            location: LocationConfig::default(),
            switch_system: true,
            switch_apps: true,
            autostart: false,
            show_tray: true,
            notify_on_switch: true,
            language: default_language(),
            manual_override_until: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocationConfig {
    pub enabled: bool,
    pub lat: f64,
    pub lon: f64,
}

impl Default for LocationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            lat: 39.9042,
            lon: 116.4074,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = config_path();
        if path.exists() {
            let s = std::fs::read_to_string(&path)?;
            let c: AppConfig = serde_json::from_str(&s)?;
            return Ok(c);
        }
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        let s = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, s)?;
        Ok(())
    }
}
