use crate::core::EventModel;

const DIRECT_MERGE_GAP_MS: u64 = 10_000;
const NOISE_EVENT_MAX_MS: u64 = 90_000;

const MAX_BRIDGE_TOTAL_MS: u64 = 60_000;
const MAX_BRIDGE_EVENTS: usize = 4;

const DOMINANT_WINDOW_MS: u64 = 4 * 60_000;
const DOMINANT_MIN_SHARE: f32 = 0.72;
const DOMINANT_IGNORE_EVENT_MS: u64 = 25_000;

pub fn merge_events(events: Vec<EventModel>) -> Vec<EventModel> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut current = events;
    current.sort_by_key(|e| e.timestamp);

    loop {
        let before = current.clone();

        current = merge_adjacent(current);
        current = merge_bridge_noise(current);
        current = merge_adjacent(current);
        current = merge_dominant_windows(current);
        current = merge_adjacent(current);

        if current == before {
            break;
        }
    }

    current
}

fn merge_adjacent(events: Vec<EventModel>) -> Vec<EventModel> {
    if events.is_empty() {
        return events;
    }

    let mut result = Vec::with_capacity(events.len());

    for event in events {
        if let Some(last) = result.last_mut() {
            if can_merge_direct(last, &event) {
                merge_into(last, &event);
                continue;
            }
        }

        result.push(event);
    }

    result
}

fn merge_bridge_noise(events: Vec<EventModel>) -> Vec<EventModel> {
    if events.len() < 3 {
        return events;
    }

    let mut result = Vec::with_capacity(events.len());
    let mut i = 0;

    while i < events.len() {
        let mut base = events[i].clone();

        let mut bridge_duration = 0;
        let mut bridge_count = 0;
        let mut found = None;

        let mut j = i + 1;

        while j < events.len() {
            let current = &events[j];

            if same_signature(&base, current) {
                found = Some(j);
                break;
            }

            if current.duration > NOISE_EVENT_MAX_MS {
                break;
            }

            bridge_duration += current.duration;
            bridge_count += 1;

            if bridge_duration > MAX_BRIDGE_TOTAL_MS {
                break;
            }

            if bridge_count > MAX_BRIDGE_EVENTS {
                break;
            }

            j += 1;
        }

        if let Some(end_idx) = found {
            for k in i + 1..=end_idx {
                merge_into(&mut base, &events[k]);
            }

            result.push(base);
            i = end_idx + 1;
        } else {
            result.push(events[i].clone());
            i += 1;
        }
    }

    result
}

fn merge_dominant_windows(events: Vec<EventModel>) -> Vec<EventModel> {
    if events.len() < 2 {
        return events;
    }

    let mut result = Vec::new();
    let mut i = 0;

    while i < events.len() {
        let window_start = events[i].timestamp;
        let window_end = window_start + DOMINANT_WINDOW_MS;

        let mut j = i;
        while j < events.len() && events[j].timestamp < window_end {
            j += 1;
        }

        if j - i <= 1 {
            result.push(events[i].clone());
            i += 1;
            continue;
        }

        let slice = &events[i..j];

        if let Some(dominant_sig) = dominant_signature(slice) {
            let mut merged: Option<EventModel> = None;

            for ev in slice {
                if signature_of(ev) == dominant_sig {
                    if let Some(ref mut m) = merged {
                        merge_into(m, ev);
                    } else {
                        merged = Some(ev.clone());
                    }
                } else if ev.duration > DOMINANT_IGNORE_EVENT_MS {
                    if let Some(m) = merged.take() {
                        result.push(m);
                    }

                    result.push(ev.clone());
                }
            }

            if let Some(m) = merged {
                result.push(m);
            }

            i = j;
        } else {
            result.push(events[i].clone());
            i += 1;
        }
    }

    result
}

fn dominant_signature(slice: &[EventModel]) -> Option<String> {
    use std::collections::HashMap;

    let mut totals: HashMap<String, u64> = HashMap::new();
    let mut total = 0u64;

    for ev in slice {
        let sig = signature_of(ev);
        *totals.entry(sig).or_default() += ev.duration;
        total += ev.duration;
    }

    if total == 0 {
        return None;
    }

    totals
        .into_iter()
        .max_by_key(|(_, duration)| *duration)
        .and_then(|(sig, duration)| {
            let share = duration as f32 / total as f32;

            if share >= DOMINANT_MIN_SHARE {
                Some(sig)
            } else {
                None
            }
        })
}

fn can_merge_direct(a: &EventModel, b: &EventModel) -> bool {
    if !same_signature(a, b) {
        return false;
    }

    if overlaps(a, b) {
        return true;
    }

    gap(a, b) <= DIRECT_MERGE_GAP_MS
}

fn overlaps(a: &EventModel, b: &EventModel) -> bool {
    end_ts(a) >= b.timestamp
}

fn same_signature(a: &EventModel, b: &EventModel) -> bool {
    signature_of(a) == signature_of(b)
}

fn signature_of(event: &EventModel) -> String {
    let event_type = format!("{:?}", event.event_type);

    match &event.window {
        Some(window) => {
            let title = window.title.clone();
            format!("{}:{}:{}", event_type, window.process_name, normalize_title(&title))
        }
        None => event_type,
    }
}

fn normalize_title(title: &str) -> String {
    title
        .trim()
        .to_lowercase()
        .chars()
        .take(80)
        .collect()
}

fn merge_into(target: &mut EventModel, incoming: &EventModel) {
    let start = target.timestamp.min(incoming.timestamp);
    let end = end_ts(target).max(end_ts(incoming));

    target.timestamp = start;
    target.duration = end - start;
}

fn end_ts(e: &EventModel) -> u64 {
    e.timestamp + e.duration
}

fn gap(a: &EventModel, b: &EventModel) -> u64 {
    b.timestamp.saturating_sub(end_ts(a))
}

pub fn merge_visual_density(
    events: Vec<EventModel>,
    px_per_hour: f64,
    min_px: f64,
) -> Vec<EventModel> {
    if events.len() < 2 {
        return events;
    }

    let ms_per_px = 3_600_000.0 / px_per_hour;
    let min_duration = (ms_per_px * min_px) as u64;

    let mut result: Vec<EventModel> = Vec::with_capacity(events.len());

    for event in events {
        if event.duration >= min_duration {
            result.push(event);
            continue;
        }

        if let Some(last) = result.last_mut() {
            merge_into(last, &event);
        } else {
            result.push(event);
        }
    }

    result
}