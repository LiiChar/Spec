use chrono::DateTime;
use dioxus::prelude::*;

use crate::{
    core::EventModel,
    lib::{
        convert_ts_to_local_date, current_ts, format_duration_short, get_process_color_gradient,
    },
};

#[component]
pub fn EventsList(events: ReadSignal<Vec<EventModel>>) -> Element {
    let summary = use_memo(move || {
        let items = events.read();
        let total_duration: u64 = items.iter().map(|event| event.duration).sum();

        let mut apps: Vec<String> = items
            .iter()
            .filter_map(|event| {
                event
                    .window
                    .as_ref()
                    .map(|window| window.process_name.clone())
            })
            .collect();
        apps.sort();
        apps.dedup();

        (items.len(), apps.len(), total_duration)
    });

    let recent_events = use_memo(move || {
        let items = events.read();
        let mut recent: Vec<EventModel> = items.iter().rev().take(50).cloned().collect();
        recent
    });

    let (event_count, app_count, total_duration) = summary();
    let total_formatted = format_duration_short(total_duration);

    rsx! {
        div {
            class: "flex flex-col gap-3 w-full h-full",

            div {
                class: "grid grid-cols-3 gap-2 px-3 py-2 bg-secondary/40 rounded-md border border-border/30 flex-shrink-0",

                div {
                    class: "flex flex-col items-center",
                    span { class: "text-muted-foreground text-xs", "События" }
                    span { class: "text-xl font-semibold text-primary", "{event_count}" }
                }

                div {
                    class: "flex flex-col items-center",
                    span { class: "text-muted-foreground text-xs", "Приложения" }
                    span { class: "text-xl font-semibold text-primary", "{app_count}" }
                }

                div {
                    class: "flex flex-col items-center",
                    span { class: "text-muted-foreground text-xs", "Время" }
                    span { class: "text-xl font-semibold text-primary", "{total_formatted}" }
                }
            }

            div {
                class: "flex flex-col gap-1.5 flex-1 overflow-auto pr-1",

                {recent_events().into_iter().map(|event| {
                    let start_dt = convert_ts_to_local_date(event.timestamp);
                    let end_dt = convert_ts_to_local_date(event.timestamp + event.duration);

                    let start_str = start_dt.format("%H:%M:%S").to_string();
                    let end_str = end_dt.format("%H:%M:%S").to_string();
                    let duration_formatted = format_duration_short(current_ts() - event.timestamp + event.duration);
                    let event_type_str = format!("{:?}", event.event_type);

                    let window_info = event.window.as_ref();
                    let process_name = window_info
                        .map(|window| window.process_name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    let window_title = window_info
                        .map(|window| window.title.clone())
                        .unwrap_or_else(|| "N/A".to_string());
                    let pid_str = window_info
                        .map(|window| window.pid.to_string())
                        .unwrap_or_else(|| "0".to_string());
                    let rect_size = window_info
                        .map(|window| format!("{}x{}", window.rect.width, window.rect.height))
                        .unwrap_or_else(|| "0x0".to_string());
                    let color_gradient = get_process_color_gradient(&process_name);

                    rsx! {
                        div {
                            class: "group relative flex flex-row items-center gap-3 px-4 py-3 bg-zinc-900/40 hover:bg-zinc-800/60 rounded-lg border border-zinc-700/30 hover:border-zinc-600/50 transition-all cursor-pointer",

                            if let Some(window) = window_info {
                                if let Some(icon) = window.icon_base64.as_deref() {
                                    img {
                                        src: icon
                                    }
                                } else {
                                    div {
                                        class: format!(
                                            "flex items-center justify-center w-10 h-10 rounded-lg bg-gradient-to-br {} text-white font-bold text-lg select-none cursor-grab active:cursor-grabbing hover:scale-110 transition-transform",
                                            color_gradient
                                        ),
                                    }
                                }
                            }

                            div {
                                class: "flex-1 flex flex-col gap-1",

                                div {
                                    class: "font-semibold text-white text-sm",
                                    "{process_name}"
                                }

                                div {
                                    class: "text-gray-400 text-xs truncate max-w-[300px]",
                                    title: "{window_title}",
                                    "{window_title}"
                                }

                                div {
                                    class: "text-gray-500 text-[11px]",
                                    "{start_str} -> {end_str}"
                                }
                            }

                            div {
                                class: "flex flex-col items-end gap-1",
                                div {
                                    class: "font-bold text-cyan-400 text-sm",
                                    "{duration_formatted} назад"
                                }
                            }

                            div {
                                class: "absolute top-full left-4 mt-1 hidden group-hover:block z-50 bg-zinc-950 text-white p-3 rounded-md shadow-xl text-xs border border-zinc-700 whitespace-nowrap",

                                div { class: "font-bold text-cyan-400 mb-2", "Детали события" }
                                div {
                                    class: "flex flex-col gap-1 text-gray-300",
                                    div { "Процесс: {process_name}" }
                                    div { "PID: {pid_str}" }
                                    div { "Размер окна: {rect_size}" }
                                    div { "Тип события: {event_type_str}" }
                                }
                            }
                        }
                    }
                })}
            }
        }
    }
}
