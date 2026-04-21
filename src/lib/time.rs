use std::time::{SystemTime};

use chrono::{DateTime, Local, NaiveDateTime, Utc};
use dioxus::desktop::wry::cookie::time::Date;

pub fn current_ts() -> u64 {
    chrono::prelude::DateTime::<chrono::Utc>::from(SystemTime::now())
        .timestamp_millis() as u64
}

pub fn convert_ts_to_date(ts: u64) -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp_millis(ts as i64).unwrap()
}

pub fn convert_ts_to_local_date(ts: u64) -> DateTime<Local> {
    let uts = convert_ts_to_date(ts);
    uts.with_timezone(&Local)
}

/// Форматирует duration (в миллисекундах) в читаемую строку
/// Примеры:
/// - 1000ms -> "1s"
/// - 65000ms -> "1m 5s"
/// - 3665000ms -> "1h 1m"
pub fn format_duration(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1000;
    
    if total_seconds == 0 {
        return "0s".to_string();
    }
    
    let hours = total_seconds / 3600;
    let remaining_after_hours = total_seconds % 3600;
    let minutes = remaining_after_hours / 60;
    let seconds = remaining_after_hours % 60;
    
    match (hours, minutes, seconds) {
        (h, m, s) if h > 0 => {
            if m > 0 {
                format!("{}h {}m", h, m)
            } else {
                format!("{}h", h)
            }
        }
        (_, m, s) if m > 0 => {
            if s > 0 {
                format!("{}m {}s", m, s)
            } else {
                format!("{}m", m)
            }
        }
        (_, _, s) => format!("{}s", s),
    }
}

/// Форматирует duration в компактный формат для отображения на UI
/// Примеры:
/// - 1000ms -> "1s"
/// - 65000ms -> "1м 5с"
/// - 3665000ms -> "1ч 1м"
pub fn format_duration_short(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1000;
    
    if total_seconds == 0 {
        return "0с".to_string();
    }
    
    let hours = total_seconds / 3600;
    let remaining_after_hours = total_seconds % 3600;
    let minutes = remaining_after_hours / 60;
    let seconds = remaining_after_hours % 60;
    
    match (hours, minutes, seconds) {
        (h, m, _) if h > 0 => format!("{}ч {}м", h, m),
        (_, m, s) if m > 0 => format!("{}м {}с", m, s),
        (_, _, s) => format!("{}с", s),
    }
}
