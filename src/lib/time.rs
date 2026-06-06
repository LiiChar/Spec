use std::time::SystemTime;

use chrono::{DateTime, Local, NaiveDateTime, Utc};

pub fn current_ts() -> u64 {
    chrono::prelude::DateTime::<chrono::Utc>::from(SystemTime::now()).timestamp_millis() as u64
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

    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{days}d"));
    }
    if hours > 0 {
        parts.push(format!("{hours}h"));
    }
    if minutes > 0 {
        parts.push(format!("{minutes}m"));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!("{seconds}s"));
    }

    parts.join(" ")
}

pub fn format_duration_short(duration_ms: u64) -> String {
    let total_seconds = duration_ms / 1000;

    if total_seconds == 0 { 
        return "0s".to_string();
    }

    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    match (days, hours, minutes, seconds) {
        (d, h, _, _) if d > 0 => {
            if h > 0 {
                format!("{d}d {h}h")
            } else {
                format!("{d}d")
            }
        }
        (_, h, m, _) if h > 0 => {
            if m > 0 {
                format!("{h}h {m}m")
            } else {
                format!("{h}h")
            }
        }
        (_, _, m, s) if m > 0 => {
            if s > 0 {
                format!("{m}m {s}s")
            } else {
                format!("{m}m")
            }
        }
        (_, _, _, s) => format!("{s}s"),
    }
}


pub fn get_start_day_ts() -> i64 {
    let now = Local::now();
    let selected_day = now.date_naive();
    selected_day
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp_millis()
}
