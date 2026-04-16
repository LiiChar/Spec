use std::time::{SystemTime};

use chrono::{DateTime, Local, NaiveDateTime, Utc};

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
