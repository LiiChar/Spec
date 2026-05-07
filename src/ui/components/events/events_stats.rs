use std::collections::HashMap;

use chrono::{Local, TimeZone};
use dioxus::prelude::*;

use crate::{
    core::{EventModel, EventType},
    lib::convert_ts_to_local_date,
    ui::{AppProvider, use_app},
};

#[component]
pub fn EventsStats() -> Element {
    let context = use_app();
    let events = context.events;

    let value = events.clone();
    let stats = use_memo(move || {
        let items = &value;
        let total_time: u64 = items.iter().map(|event| event.duration).sum();

        let active_time: u64 = items
            .iter()
            .filter(|e| e.event_type != EventType::Idle)
            .map(|e| e.duration)
            .sum();

        let idle_time: u64 = total_time - active_time;

        let mut app_stats: HashMap<String, (u64, u64, Option<String>)> = HashMap::new(); // (active, idle)

        for event in items.iter() {
            if let Some(window) = &event.window {
                let app = &window.process_name;
                let (active, idle, icon) =
                    app_stats
                        .entry(app.clone())
                        .or_insert((0, 0, window.icon_base64.clone()));
                if event.event_type == EventType::Idle {
                    *idle += event.duration;
                } else {
                    *active += event.duration;
                }
            }
        }

        let mut app_list: Vec<(String, u64, u64, Option<String>)> = app_stats
            .into_iter()
            .map(|(name, (active, idle, icon))| (name, active, idle, icon))
            .collect();
        app_list.sort_by(|a, b| (b.1 + b.2).cmp(&(a.1 + a.2))); // sort by total time desc

        let num_events = items.len();
        let num_apps = app_list.len();

        let avg_event_duration = if num_events > 0 {
            total_time / num_events as u64
        } else {
            0
        };
        let most_used_app = app_list.first().cloned();

        (
            total_time,
            active_time,
            idle_time,
            app_list,
            num_events,
            num_apps,
            avg_event_duration,
            most_used_app,
        )
    });

    let (
        total_time,
        active_time,
        idle_time,
        app_list,
        num_events,
        num_apps,
        avg_event_duration,
        most_used_app,
    ) = stats();

    // Helper to format time
    let format_time = |t: u64| {
        let hours = t / 3600000;
        let minutes = (t % 3600000) / 60000;
        let seconds = (t % 60000) / 1000;
        format!("{}h {}m {}s", hours, minutes, seconds)
    };

    let fmt_start_date = events
        .read()
        .first()
        .map(|e| {
            let date = convert_ts_to_local_date(e.timestamp);
            date.format("%d.%m.%Y %H:%M:%S").to_string()
        })
        .unwrap_or_default();
    let fmt_end_date = events
        .read()
        .last()
        .map(|e| {
            let date = convert_ts_to_local_date(e.timestamp);
            date.format("%d.%m.%Y %H:%M:%S").to_string()
        })
        .unwrap_or_default();

    rsx! {
        div {
            class: "flex flex-col gap-3 rounded-md",
            div {
                class: "rounded-md shadow-sm p-3  border border-border/30",
                h3 { class: "text-lg font-semibold mb-3 flex flex-col",
                    span {
                        "Статистика"
                    }
                    span {
                        class: "text-xs text-muted-foreground",
                        "{fmt_start_date} - {fmt_end_date}"
                    }
                }
                div { class: "grid grid-cols-2 gap-3",
                    div {
                        class: "text-xs text-muted-foreground", "Общее время"
                        div { class: "text-base font-semibold text-foreground", "{format_time(total_time)}" }
                    }
                    div {
                        class: "text-xs text-muted-foreground", "Активное время"
                        div { class: "text-base font-semibold text-primary", "{format_time(active_time)}" }
                    }
                    div {
                        class: "text-xs text-muted-foreground", "Бездействие"
                        div { class: "text-base font-semibold text-foreground", "{format_time(idle_time)}" }
                    }
                    div {
                        class: "text-xs text-muted-foreground", "Количество событий"
                        div { class: "text-base font-semibold text-foreground", "{num_events}" }
                    }
                    div {
                        class: "text-xs text-muted-foreground", "Средняя продолжительность"
                        div { class: "text-base font-semibold text-foreground", "{format_time(avg_event_duration)}" }
                    }
                    if let Some((app, active, idle, icon)) = most_used_app {
                        div {
                            class: "text-xs text-muted-foreground", "Самое используемое"
                            div { class: "text-base font-semibold text-foreground", "{app}" }
                            div { class: "text-xs text-muted-foreground", "Актив: {format_time(active)}, Idle: {format_time(idle)}" }
                        }
                    }
                }
            }
            div {
                class: "rounded-md shadow-sm p-3  border border-border/30",
                h3 { class: "text-sm font-semibold mb-3", "Приложения ({num_apps})" }
                div { class: "flex flex-col gap-1",
                    for (app, active, idle, icon) in app_list {
                        div { class: "flex justify-between items-center p-1.5 rounded hover:bg-secondary/30 transition-colors",
                            div {
                                class: "flex gap-2 items-center",
                                if let Some(icon) = icon {
                                    img {
                                        class: "w-5 h-5 rounded",
                                        src: icon
                                    }
                                }
                                span { class: "text-xs font-medium text-foreground truncate max-w-[150px]", "{app}" }
                            }
                            div { class: "flex flex-col text-end",
                                span { class: "text-xs text-foreground", "{format_time(active)}" }
                                span { class: "text-[10px] text-muted-foreground", "{format_time(idle)} idle" }
                            }
                        }
                    }
                }
            }
        }
    }
}
