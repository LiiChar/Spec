
use std::collections::HashMap;

use crate::core::{EventModel, EventType};

#[derive(Debug, Clone)]
pub struct AppStat {
    pub name: String,
    pub active_time: u64,
    pub idle_time: u64,
    pub total_time: u64,
    pub active_percent: f32,
    pub icon: Option<String>,
    pub event: EventModel
}

#[derive(Debug, Clone)]
pub struct EventTypeStat {
    pub event_type: EventType,
    pub count: usize,
    pub total_time: u64,
    pub percent: f32,
}

#[derive(Debug, Clone)]
pub struct EventStats {
    pub total_time: u64,
    pub active_time: u64,
    pub idle_time: u64,

    pub active_percent: f32,
    pub idle_percent: f32,

    pub num_events: usize,
    pub num_apps: usize,

    pub avg_event_duration: u64,
    pub avg_active_event_duration: u64,

    pub longest_event: Option<EventModel>,
    pub most_used_app: Option<AppStat>,

    pub app_list: Vec<AppStat>,
    pub event_types: Vec<EventTypeStat>,
}


pub fn event_stats(items: Vec<EventModel>) -> EventStats {
    let total_time: u64 = items.iter().map(|event| event.duration).sum();

    let active_time: u64 = items
        .iter()
        .filter(|e| e.event_type != EventType::Idle)
        .map(|e| e.duration)
        .sum();

    let idle_time = total_time.saturating_sub(active_time);

    let active_percent = if total_time > 0 {
        active_time as f32 * 100.0 / total_time as f32
    } else {
        0.0
    };

    let idle_percent = if total_time > 0 {
        idle_time as f32 * 100.0 / total_time as f32
    } else {
        0.0
    };

    let mut app_stats: HashMap<String, (u64, u64, EventModel, Option<String>)> = HashMap::new();

    for event in items.iter() {
        if let Some(window) = &event.window {
            let entry = app_stats
                .entry(window.process_name.clone())
                .or_insert((0, 0, event.clone(), window.icon_base64.clone()));

            if event.event_type == EventType::Idle {
                entry.1 += event.duration;
            } else {
                entry.0 += event.duration;
            }
        }
    }

    let mut app_list: Vec<AppStat> = app_stats
        .into_iter()
        .map(|(name, (active, idle,event, icon))| {
            let total = active + idle;
            let active_percent = if total > 0 {
                active as f32 * 100.0 / total as f32
            } else {
                0.0
            };

            AppStat {
                name,
                active_time: active,
                idle_time: idle,
                total_time: total,
                active_percent,
                icon,
                event
            }
        })
        .collect();

    app_list.sort_by(|a, b| b.total_time.cmp(&a.total_time));

    let num_events = items.len();
    let num_apps = app_list.len();

    let avg_event_duration = if num_events > 0 {
        total_time / num_events as u64
    } else {
        0
    };

    let active_events_count = items
        .iter()
        .filter(|e| e.event_type != EventType::Idle)
        .count();

    let avg_active_event_duration = if active_events_count > 0 {
        active_time / active_events_count as u64
    } else {
        0
    };

    let longest_event = items.iter().max_by_key(|e| e.duration).cloned();

    let most_used_app = app_list.first().cloned();

    let mut type_map: HashMap<EventType, (usize, u64)> = HashMap::new();

    for event in items.iter() {
        let entry = type_map
            .entry(event.event_type.clone())
            .or_insert((0, 0));

        entry.0 += 1;
        entry.1 += event.duration;
    }

    let mut event_types: Vec<EventTypeStat> = type_map
        .into_iter()
        .map(|(event_type, (count, duration))| {
            let percent = if total_time > 0 {
                duration as f32 * 100.0 / total_time as f32
            } else {
                0.0
            };

            EventTypeStat {
                event_type,
                count,
                total_time: duration,
                percent,
            }
        })
        .collect();

    event_types.sort_by(|a, b| b.total_time.cmp(&a.total_time));

    EventStats {
        total_time,
        active_time,
        idle_time,

        active_percent,
        idle_percent,

        num_events,
        num_apps,

        avg_event_duration,
        avg_active_event_duration,

        longest_event,
        most_used_app,

        app_list,
        event_types,
    }
}