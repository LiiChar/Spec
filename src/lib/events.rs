use crate::core::EventModel;

const DIRECT_MERGE_GAP_MS: u64 = 10_000;
const BRIDGE_GAP_MS: u64 = 15_000;
const NOISE_EVENT_MAX_MS: u64 = 90_000;
const MIN_STABLE_SEGMENT_MS: u64 = 180_000;

pub fn merge_events(events: &[EventModel]) -> Vec<EventModel> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut current = events.to_vec();
    current.sort_by_key(|e| e.timestamp);

    loop {
        let before_len = current.len();

        current = merge_adjacent(current);
        current = merge_bridge_noise(current);

        // если больше ничего не меняется — выходим
        if current.len() == before_len {
            break;
        }
    }

    current
}

fn merge_adjacent(events: Vec<EventModel>) -> Vec<EventModel> {
    let mut result: Vec<EventModel> = Vec::with_capacity(events.len());

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
        let mut merged_any = false;

        // пытаемся схлопнуть цепочку, а не только один middle
        if i + 2 < events.len() {
            let mut base = events[i].clone();
            let mut j = i;

            while j + 2 < events.len() {
                let middle = &events[j + 1];
                let next = &events[j + 2];

                if should_absorb_middle(&base, middle, next) {
                    merge_into(&mut base, middle);
                    merge_into(&mut base, next);
                    j += 2;
                    merged_any = true;
                } else {
                    break;
                }
            }

            if merged_any {
                result.push(base);
                i = j + 1;
                continue;
            }
        }

        result.push(events[i].clone());
        i += 1;
    }

    result
}

fn should_absorb_middle(prev: &EventModel, middle: &EventModel, next: &EventModel) -> bool {
    if !same_signature(prev, next) {
        return false;
    }

    if same_signature(prev, middle) || same_signature(middle, next) {
        return false;
    }

    if middle.duration > NOISE_EVENT_MAX_MS {
        return false;
    }

    if prev.duration < MIN_STABLE_SEGMENT_MS && next.duration < MIN_STABLE_SEGMENT_MS {
        return false;
    }

    let gap_before = gap(prev, middle);
    let gap_after = gap(middle, next);

    gap_before <= BRIDGE_GAP_MS && gap_after <= BRIDGE_GAP_MS
}

fn can_merge_direct(a: &EventModel, b: &EventModel) -> bool {
    if b.timestamp < a.timestamp {
        return false;
    }

    if !same_signature(a, b) {
        return false;
    }

    gap(a, b) <= DIRECT_MERGE_GAP_MS
}

fn same_signature(a: &EventModel, b: &EventModel) -> bool {
    if a.event_type != b.event_type {
        return false;
    }

    match (&a.window, &b.window) {
        (Some(l), Some(r)) => l.process_name == r.process_name,
        (None, None) => true,
        _ => false,
    }
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