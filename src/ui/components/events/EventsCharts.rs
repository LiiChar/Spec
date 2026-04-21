use std::collections::HashMap;

use chrono::{Local, Timelike};
use dioxus::prelude::*;

use crate::{
    core::EventModel,
    ui::AppContext,
};

#[derive(PartialEq, Clone)]
pub enum ChartType {
    Bar,
    Timeline,
}

#[derive(Props, PartialEq, Clone)]
pub struct EventsChartsProps {
    charts: Vec<ChartType>,
}

const COLORS: &[&str] = &[
    "#3B82F6", "#8B5CF6", "#EC4899", "#F59E0B", "#10B981",
    "#06B6D4", "#6366F1", "#14B8A6", "#F97316", "#6B21A8",
];

fn get_color(index: usize) -> &'static str {
    COLORS[index % COLORS.len()]
}

#[component]
pub fn EventsCharts(props: EventsChartsProps) -> Element {
    let context = use_context::<AppContext>();
    let events = context.events;

    let value = events.clone();
    let summary = use_memo(move || {
        let items = &value;
        let total_time: u64 = items.iter().map(|event| event.duration).sum();

        let mut app_times: HashMap<String, u64> = HashMap::new();
        for event in items.iter() {
            if let Some(window) = &event.window {
                *app_times.entry(window.process_name.clone()).or_default() += event.duration;
            }
        }

        let unique_apps = app_times.len();
        let mut top_apps: Vec<(String, u64)> = app_times.into_iter().collect();
        top_apps.sort_by(|a, b| b.1.cmp(&a.1));
        top_apps.truncate(10);

        (items.len(), total_time, unique_apps, top_apps)
    });

    let summary = summary();

    rsx! {
        div {
            class: "flex flex-col gap-6 p-6 bg-gradient-to-br from-slate-50 to-slate-100 rounded-lg",
            {props.charts.iter().map(|chart| {
                match chart {
                    ChartType::Bar => rsx! {
                        div {
                            class: "bg-white rounded-lg shadow-sm border border-slate-200 p-6",
                            h3 { 
                                class: "text-xl font-semibold text-slate-900 mb-4",
                                "Top Applications"
                            }
                            BarChart { data: summary.3.clone() }
                        }
                    },
                    ChartType::Timeline => rsx! {
                        div {
                            class: "bg-white rounded-lg shadow-sm border border-slate-200 p-6",
                            h3 { 
                                class: "text-xl font-semibold text-slate-900 mb-4",
                                "Daily Timeline"
                            }
                            TimelineChart { events: events }
                        }
                    },
                }
            })}
        }
    }
}

#[component]
fn BarChart(data: Vec<(String, u64)>) -> Element {
    let max_time = data.iter().map(|(_, time)| *time).max().unwrap_or(1);
    let mut sorted_data = data.clone();
    sorted_data.sort_by(|a, b| a.1.cmp(&b.1));

    rsx! {
        div {
            class: "w-full space-y-3",
            {sorted_data.iter().enumerate().map(|(i, (app, time))| {
                let percentage = (*time as f64 / max_time as f64) * 100.0;
                let hours = *time / (1000 * 60 * 60);
                let minutes = (*time % (1000 * 60 * 60)) / (1000 * 60);
                let color = get_color(i);

                rsx! {
                    div {
                        class: "flex items-center gap-3",
                        div {
                            class: "w-32 text-sm font-medium text-slate-700 truncate",
                            "{app}"
                        }
                        div {
                            class: "flex-1",
                            div {
                                class: "h-8 rounded-md overflow-hidden bg-slate-100 shadow-inner",
                                style: "background: linear-gradient(90deg, {color} 0%, {color}dd 100%);",
                                div {
                                    class: "h-full w-full transition-all duration-500",
                                    style: "width: {percentage}%;",
                                }
                            }
                        }
                        div {
                            class: "text-sm font-semibold text-slate-600 min-w-fit",
                            "{hours}h {minutes}m"
                        }
                    }
                }
            })}
        }
    }
}

#[component]
fn TimelineChart(events: ReadSignal<Vec<EventModel>>) -> Element {
    let items = events.read().clone();
    
    let mut hour_events: HashMap<u32, Vec<EventModel>> = HashMap::new();
    for event in items.iter() {
        let hour = chrono::DateTime::from_timestamp_millis(event.timestamp as i64)
            .map(|dt| dt.with_timezone(&Local).hour())
            .unwrap_or(0);
        
        hour_events.entry(hour).or_default().push(event.clone());
    }

    let total_time: u64 = items.iter().map(|e| e.duration).sum();

    rsx! {
        div {
            class: "w-full",
            div {
                class: "flex items-end gap-2 h-64 bg-gradient-to-t from-slate-50 to-transparent rounded-md p-4 border border-slate-200",
                {(0..24).map(|hour| {
                    let hour_total = hour_events.get(&hour)
                        .map(|evts| evts.iter().map(|e| e.duration).sum::<u64>())
                        .unwrap_or(0);
                    
                    let height_percent = if total_time > 0 {
                        (hour_total as f64 / total_time as f64) * 100.0
                    } else {
                        0.0
                    };

                    let has_events = hour_total > 0;
                    let color = if has_events { "#3B82F6" } else { "#E2E8F0" };

                    rsx! {
                        div {
                            class: "flex-1 flex flex-col items-center justify-end gap-1",
                            div {
                                class: "w-full rounded-t-md transition-all duration-300 hover:shadow-lg",
                                style: "height: {height_percent}%; background-color: {color}; min-height: 2px;",
                                title: "{hour}:00 - {hour_total}ms"
                            }
                            div {
                                class: "text-xs font-medium text-slate-600 mt-1",
                                "{hour}"
                            }
                        }
                    }
                })}
            }
            div {
                class: "mt-4 grid grid-cols-4 gap-4 text-sm",
                div {
                    class: "p-3 bg-blue-50 rounded-lg border border-blue-200",
                    p { class: "text-blue-900 font-semibold", "Total" }
                    p { 
                        class: "text-blue-700 font-bold text-lg",
                        {
                            let total_ms = total_time;
                            let h = total_ms / (1000 * 60 * 60);
                            let m = (total_ms % (1000 * 60 * 60)) / (1000 * 60);
                            format!("{}h {}m", h, m)
                        }
                    }
                }
                div {
                    class: "p-3 bg-purple-50 rounded-lg border border-purple-200",
                    p { class: "text-purple-900 font-semibold", "Peak" }
                    p { 
                        class: "text-purple-700 font-bold text-lg",
                        {
                            let peak = hour_events.values()
                                .map(|evts| evts.iter().map(|e| e.duration).sum::<u64>())
                                .max()
                                .unwrap_or(0);
                            let h = peak / (1000 * 60 * 60);
                            let m = (peak % (1000 * 60 * 60)) / (1000 * 60);
                            format!("{}h {}m", h, m)
                        }
                    }
                }
                div {
                    class: "p-3 bg-green-50 rounded-lg border border-green-200",
                    p { class: "text-green-900 font-semibold", "Events" }
                    p { 
                        class: "text-green-700 font-bold text-lg",
                        "{items.len()}"
                    }
                }
                div {
                    class: "p-3 bg-orange-50 rounded-lg border border-orange-200",
                    p { class: "text-orange-900 font-semibold", "Avg/hour" }
                    p { 
                        class: "text-orange-700 font-bold text-lg",
                        {
                            let hours_active = (0..24).filter(|h| hour_events.contains_key(h)).count().max(1);
                            let avg_per_hour = items.len() / hours_active;
                            format!("{}", avg_per_hour)
                        }
                    }
                }
            }
        }
    }
}