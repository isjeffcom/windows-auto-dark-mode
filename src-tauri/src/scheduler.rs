use chrono::{Local, NaiveTime, Timelike, Utc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::config::{AppConfig, ScheduleMode};
use crate::AppState;

use sunrise_sunset_calculator::{SunriseSunsetParameters, SunriseSunsetResult};

static RUNNING: AtomicBool = AtomicBool::new(true);

fn parse_time(s: &str) -> Option<NaiveTime> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() >= 2 {
        let h: u32 = parts[0].trim().parse().ok()?;
        let m: u32 = parts[1].trim().parse().ok()?;
        NaiveTime::from_hms_opt(h, m, 0)
    } else {
        None
    }
}

fn should_be_light_now(config: &AppConfig) -> Option<bool> {
    let now = Local::now().time();
    match config.mode {
        ScheduleMode::Fixed => {
            let sunrise = parse_time(&config.sunrise_time)?;
            let sunset = parse_time(&config.sunset_time)?;
            if sunrise <= sunset {
                Some(now >= sunrise && now < sunset)
            } else {
                Some(now >= sunrise || now < sunset)
            }
        }
        ScheduleMode::Location if config.location.enabled => {
            let ts = Local::now().timestamp();
            let params = SunriseSunsetParameters::new(ts, config.location.lat, config.location.lon);
            let res: SunriseSunsetResult = params.calculate().ok()?;
            let sunrise = chrono::DateTime::from_timestamp(res.rise, 0)?
                .with_timezone(&Local)
                .time();
            let sunset = chrono::DateTime::from_timestamp(res.set, 0)?
                .with_timezone(&Local)
                .time();
            Some(now >= sunrise && now < sunset)
        }
        _ => None,
    }
}

pub fn compute_sun_times(config: &AppConfig) -> Result<Option<super::SunTimes>, String> {
    if config.mode != ScheduleMode::Location || !config.location.enabled {
        return Ok(None);
    }
    let ts = Local::now().timestamp();
    let params = SunriseSunsetParameters::new(ts, config.location.lat, config.location.lon);
    let res = params.calculate().map_err(|e| e.to_string())?;
    let sunrise_dt = chrono::DateTime::from_timestamp(res.rise, 0)
        .ok_or("invalid sunrise timestamp")?
        .with_timezone(&Local);
    let sunset_dt = chrono::DateTime::from_timestamp(res.set, 0)
        .ok_or("invalid sunset timestamp")?
        .with_timezone(&Local);
    let (next_switch, next_is_dark) = if Local::now().time() < sunrise_dt.time() {
        (sunrise_dt.format("%H:%M").to_string(), false)
    } else if Local::now().time() < sunset_dt.time() {
        (sunset_dt.format("%H:%M").to_string(), true)
    } else {
        let tomorrow_ts = (Utc::now() + chrono::Duration::days(1)).timestamp();
        let params2 = SunriseSunsetParameters::new(
            tomorrow_ts,
            config.location.lat,
            config.location.lon,
        );
        let res2 = params2.calculate().map_err(|e| e.to_string())?;
        let sr_dt = chrono::DateTime::from_timestamp(res2.rise, 0)
            .ok_or("invalid timestamp")?
            .with_timezone(&Local);
        (sr_dt.format("%H:%M").to_string(), false)
    };
    Ok(Some(super::SunTimes {
        sunrise: sunrise_dt.format("%H:%M").to_string(),
        sunset: sunset_dt.format("%H:%M").to_string(),
        next_switch,
        next_switch_is_dark: next_is_dark,
    }))
}

/// Returns the next "boundary" timestamp (next sunrise or sunset) when the schedule would naturally switch.
/// Used to set manual_override_until so we don't auto-switch until the next period.
pub fn get_next_boundary_timestamp(config: &AppConfig) -> Option<i64> {
    let now = Local::now();
    match config.mode {
        ScheduleMode::Fixed => {
            let sunrise = parse_time(&config.sunrise_time)?;
            let sunset = parse_time(&config.sunset_time)?;
            let today = now.date_naive();
            let sunrise_today = today.and_time(sunrise).and_local_timezone(Local).earliest()?;
            let sunset_today = today.and_time(sunset).and_local_timezone(Local).earliest()?;
            if now < sunrise_today {
                Some(sunrise_today.timestamp())
            } else if now < sunset_today {
                Some(sunset_today.timestamp())
            } else {
                let tomorrow = today.succ_opt()?;
                let sunrise_tomorrow = tomorrow.and_time(sunrise).and_local_timezone(Local).earliest()?;
                Some(sunrise_tomorrow.timestamp())
            }
        }
        ScheduleMode::Location if config.location.enabled => {
            let ts = now.timestamp();
            let params = SunriseSunsetParameters::new(ts, config.location.lat, config.location.lon);
            let res = params.calculate().ok()?;
            let sunrise_dt = chrono::DateTime::from_timestamp(res.rise, 0)?.with_timezone(&Local);
            let sunset_dt = chrono::DateTime::from_timestamp(res.set, 0)?.with_timezone(&Local);
            if now < sunrise_dt {
                Some(sunrise_dt.timestamp())
            } else if now < sunset_dt {
                Some(sunset_dt.timestamp())
            } else {
                let tomorrow_ts = (Utc::now() + chrono::Duration::days(1)).timestamp();
                let params2 = SunriseSunsetParameters::new(tomorrow_ts, config.location.lat, config.location.lon);
                let res2 = params2.calculate().ok()?;
                let sr = chrono::DateTime::from_timestamp(res2.rise, 0)?.with_timezone(&Local);
                Some(sr.timestamp())
            }
        }
        _ => {
            get_next_boundary_timestamp(&AppConfig {
                mode: ScheduleMode::Fixed,
                ..config.clone()
            })
        }
    }
}

pub fn get_next_switch_time(config: &AppConfig) -> Result<Option<(String, bool)>, String> {
    let now = Local::now().time();
    match config.mode {
        ScheduleMode::Fixed => {
            let sunrise = parse_time(&config.sunrise_time).unwrap_or(NaiveTime::from_hms_opt(7, 0, 0).unwrap());
            let sunset = parse_time(&config.sunset_time).unwrap_or(NaiveTime::from_hms_opt(19, 0, 0).unwrap());
            if now < sunrise {
                Ok(Some((format!("{:02}:{:02}", sunrise.hour(), sunrise.minute()), false)))
            } else if now < sunset {
                Ok(Some((format!("{:02}:{:02}", sunset.hour(), sunset.minute()), true)))
            } else {
                Ok(Some((format!("{:02}:{:02}", sunrise.hour(), sunrise.minute()), false)))
            }
        }
        ScheduleMode::Location if config.location.enabled => {
            if let Some(Some(st)) = compute_sun_times(config).ok() {
                return Ok(Some((st.next_switch, st.next_switch_is_dark)));
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn apply_theme_if_needed(config: &AppConfig) {
    if !config.enabled {
        return;
    }
    if let Some(until_ts) = config.manual_override_until {
        if Local::now().timestamp() < until_ts {
            return;
        }
    }
    let Some(should_light) = should_be_light_now(config) else { return };
    #[cfg(windows)]
    {
        if let Ok(current) = crate::theme::get_theme_state() {
            let current_light = current.apps_light && current.system_light;
            if current_light != should_light {
                let _ = crate::theme::set_theme(
                    should_light,
                    config.switch_system,
                    config.switch_apps,
                );
            }
        }
    }
}

pub struct Scheduler {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Scheduler {
    pub fn start(config: AppConfig) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let handle = thread::spawn(move || {
            RUNNING.store(true, Ordering::SeqCst);
            let mut config = config;
            loop {
                if stop_clone.load(Ordering::SeqCst) {
                    break;
                }
                // Load config first so manual_override_until (and other changes) are applied before we decide to switch
                if let Ok(c) = AppConfig::load() {
                    config = c;
                }
                apply_theme_if_needed(&config);
                for _ in 0..60 {
                    if stop_clone.load(Ordering::SeqCst) {
                        return;
                    }
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        RUNNING.store(false, Ordering::SeqCst);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

pub fn restart_scheduler(state: &AppState, config: &AppConfig) -> Result<(), String> {
    let mut sched = state.scheduler.lock().map_err(|e| e.to_string())?;
    if let Some(mut s) = sched.take() {
        s.stop();
    }
    *sched = Some(Scheduler::start(config.clone()));
    Ok(())
}
