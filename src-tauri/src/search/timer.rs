use super::SearchResultItem;
use chrono::{Local, Timelike};
use regex::Regex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct TimerManager;

impl TimerManager {
    pub fn parse_command(q: &str, app: Option<AppHandle>) -> Option<SearchResultItem> {
        let trimmed = q.trim();

        if trimmed.starts_with(">timer ") || trimmed.starts_with("timer ") {
            let rest = if trimmed.starts_with(">timer ") {
                &trimmed[7..]
            } else {
                &trimmed[6..]
            };
            return Self::parse_timer(rest.trim(), app);
        }

        if trimmed.starts_with(">remind ") || trimmed.starts_with("remind ") {
            let rest = if trimmed.starts_with(">remind ") {
                &trimmed[8..]
            } else {
                &trimmed[7..]
            };
            return Self::parse_reminder(rest.trim(), app);
        }

        None
    }

    fn parse_timer(spec: &str, app: Option<AppHandle>) -> Option<SearchResultItem> {
        let parts: Vec<&str> = spec.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let dur_str = parts[0];
        let label = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            "Timer".to_string()
        };

        let secs = parse_duration_seconds(dur_str)?;

        if let Some(handle) = app {
            let label_clone = label.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(secs));
                fire_alarm(handle, &label_clone);
            });
        }

        Some(SearchResultItem {
            id: format!("timer_{}_{}", secs, label),
            title: format!("Timer set for {} ({})", dur_str, label),
            subtitle: Some(format!("Fires in {} seconds with alert sound", secs)),
            category: "Time & Reminders".into(),
            item_type: "calc".into(),
            card_type: Some("math".into()),
            path: None,
            action_hint: Some("↵ Confirm Timer".into()),
            score: 1200,
            snippet: None,
            admin_path: None,
        })
    }

    fn parse_reminder(spec: &str, app: Option<AppHandle>) -> Option<SearchResultItem> {
        let parts: Vec<&str> = spec.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let time_str = parts[0];
        let label = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            "Reminder".to_string()
        };

        let secs = parse_clock_seconds(time_str)?;

        if let Some(handle) = app {
            let label_clone = label.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(secs));
                fire_alarm(handle, &label_clone);
            });
        }

        Some(SearchResultItem {
            id: format!("remind_{}_{}", time_str, label),
            title: format!("Reminder set for {} ({})", time_str, label),
            subtitle: Some(format!("Fires in {} seconds with notification popup", secs)),
            category: "Time & Reminders".into(),
            item_type: "calc".into(),
            card_type: Some("math".into()),
            path: None,
            action_hint: Some("↵ Confirm Reminder".into()),
            score: 1200,
            snippet: None,
            admin_path: None,
        })
    }
}

fn parse_duration_seconds(spec: &str) -> Option<u64> {
    let re = Regex::new(r"(?i)^(\d+)\s*(s|sec|m|min|h|hr)?$").ok()?;
    let caps = re.captures(spec)?;
    let val: u64 = caps.get(1)?.as_str().parse().ok()?;
    let unit = caps.get(2).map(|u| u.as_str().to_lowercase());

    match unit.as_deref() {
        Some("s") | Some("sec") | None => Some(val),
        Some("m") | Some("min") => Some(val * 60),
        Some("h") | Some("hr") => Some(val * 3600),
        _ => Some(val),
    }
}

fn parse_clock_seconds(spec: &str) -> Option<u64> {
    let re = Regex::new(r"^(\d{1,2}):(\d{2})$").ok()?;
    let caps = re.captures(spec)?;
    let h: u32 = caps.get(1)?.as_str().parse().ok()?;
    let m: u32 = caps.get(2)?.as_str().parse().ok()?;

    if h > 23 || m > 59 {
        return None;
    }

    let now = Local::now();
    let current_h = now.hour();
    let current_m = now.minute();
    let current_s = now.second();

    let now_secs = current_h * 3600 + current_m * 60 + current_s;
    let target_secs = h * 3600 + m * 60;

    let delta = if target_secs > now_secs {
        target_secs - now_secs
    } else {
        // Next day
        (86400 - now_secs) + target_secs
    };

    Some(delta as u64)
}

fn fire_alarm(app: AppHandle, label: &str) {
    #[cfg(target_os = "windows")]
    unsafe {
        extern "system" {
            fn MessageBeep(u_type: u32) -> i32;
        }
        MessageBeep(0x40);
    }

    let _ = app.emit("timer-alert", label);
}
