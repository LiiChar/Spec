use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};

use crate::{core::EventModel, lib::convert_ts_to_local_date};


pub fn group_by_hours(events: &[EventModel]) -> HashMap<u32, HashMap<u32, Vec<EventModel>>> {
    let mut result: HashMap<u32, HashMap<u32, Vec<EventModel>>> = HashMap::new();

    for event in events {
        let mut current = convert_ts_to_local_date(event.timestamp);
        let end = convert_ts_to_local_date(event.timestamp + event.duration);

        while current < end {
            let next_hour = current
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                + chrono::Duration::hours(1);

            let slice_end = if next_hour < end { next_hour } else { end };
            let duration = (slice_end.timestamp_millis() - current.timestamp_millis()) as u64;

            result
                .entry(current.day())
                .or_default()
                .entry(current.hour())
                .or_default()
                .push(EventModel {
                    window: event.window.clone(),
                    event_type: event.event_type.clone(),
                    timestamp: current.timestamp_millis() as u64,
                    duration,
                });

            current = slice_end;
        }
    }
    result
}

#[derive(Clone, PartialEq, Debug)]
pub struct Segment {
  pub day: DateTime<Local>,
  pub start: u32,
  pub end: u32,
  pub group: Vec<EventModel>,
  pub has_events: bool,
}

pub fn group_by_segments(events: &[EventModel]) -> Vec<Segment> {
    if events.is_empty() {
        return Vec::new();
    }

    // --- 1. разбиваем по (день, час) ---
    let mut map: HashMap<(i64, u32), Vec<EventModel>> = HashMap::new();

    for event in events {
        let mut current = convert_ts_to_local_date(event.timestamp);
        let end = convert_ts_to_local_date(event.timestamp + event.duration);

        while current < end {
            let next_hour = current
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                + Duration::hours(1);

            let slice_end = if next_hour < end { next_hour } else { end };
            let duration = (slice_end.timestamp_millis() - current.timestamp_millis()) as u64;

            let day_ts = current
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp_millis();

            map.entry((day_ts, current.hour()))
                .or_default()
                .push(EventModel {
                    window: event.window.clone(),
                    event_type: event.event_type.clone(),
                    timestamp: current.timestamp_millis() as u64,
                    duration,
                });

            current = slice_end;
        }
    }

    // --- 2. нормализуем в timeline (с пустыми часами!) ---
    let mut timeline: Vec<(i64, u32, Vec<EventModel>)> = Vec::new();

    let mut days: Vec<i64> = map.keys().map(|(d, _)| *d).collect();
    days.sort();
    days.dedup();

    for day in days {
        for hour in 0..24 {
            let events = map.remove(&(day, hour)).unwrap_or_default();
            timeline.push((day, hour, events));
        }
    }

    timeline.sort_by_key(|(day, hour, _)| (*day, *hour));

    // --- 3. сегментация ---
    let mut segments = Vec::new();

    let (mut current_day, mut current_start, mut current_events) = {
        let (d, h, e) = timeline[0].clone();
        (d, h, e)
    };

    let mut current_end = current_start;
    let mut current_has_events = !current_events.is_empty();

    for (day, hour, events) in timeline.into_iter().skip(1) {
        let has_events = !events.is_empty();

        let is_same = day == current_day
            && hour == current_end + 1
            && (
                // объединяем только если ОБА пустые
                (!has_events && !current_has_events)
            );

        if is_same {
            current_end = hour;
            current_events.extend(events);
        } else {
            segments.push(Segment {
                day: convert_ts_to_local_date(current_day as u64),
                start: current_start,
                end: current_end,
                group: current_events.clone(),
                has_events: current_has_events,
            });

            current_day = day;
            current_start = hour;
            current_end = hour;
            current_events = events;
            current_has_events = has_events;
        }
    }

    // последний сегмент
    segments.push(Segment {
        day: convert_ts_to_local_date(current_day as u64),
        start: current_start,
        end: current_end,
        group: current_events,
        has_events: current_has_events,
    });

    segments
}


pub fn y_to_timestamp(
    y: f64,
    segments: &[Segment],
    selected_hour: Option<u32>,
    day_start: u64,
    base: u32,
    expanded: u32,
) -> u64 {
    let mut acc: f64 = 0.0;

    for seg in segments {
        let size = if seg.has_events {
            match selected_hour {
                Some(hour)
                    if hour >= seg.start && hour <= seg.end =>
                {
                    expanded as f64
                }
                _ => base as f64,
            }
        } else {
            base as f64
        };

        if y < acc + size {
            let progress = (y - acc) / size;

            let start_ts =
                day_start + seg.start as u64 * 3_600_000;

            let end_ts =
                day_start + (seg.end as u64 + 1) * 3_600_000;

            let segment_duration = end_ts - start_ts;

            return start_ts
                + (progress * segment_duration as f64) as u64;
        }

        acc += size;
    }

    day_start + 86_400_000
}

pub fn timestamp_to_y(
    ts: u64,
    segments: &[Segment],
    selected_hour: Option<u32>,
    day_start: u64,
    base: u32,
    expanded: u32,
) -> f64 {
    let mut y = 0.0;

    for seg in segments {
        let size = if seg.has_events {
            match selected_hour {
                Some(hour)
                    if hour >= seg.start && hour <= seg.end =>
                {
                    expanded as f64
                }
                _ => base as f64,
            }
        } else {
            base as f64
        };

        let start_ts =
            day_start + seg.start as u64 * 3_600_000;

        let end_ts =
            day_start + (seg.end as u64 + 1) * 3_600_000;

        if ts >= start_ts && ts < end_ts {
            let progress =
                (ts - start_ts) as f64 / (end_ts - start_ts) as f64;

            return y + progress * size;
        }

        y += size;
    }

    y
}