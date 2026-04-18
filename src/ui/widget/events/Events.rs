use dioxus::prelude::*;
use chrono::{Datelike, Timelike};

use crate::{core::EventModel, lib::convert_ts_to_local_date, ui::{EventsList, EventsTimeline, EventsWeek, TimelineOrientation}};

#[derive(PartialEq, Clone)]
pub enum ViewMode {
    Timeline,
    List,
    Stats,
}

#[component]
pub fn Events(events: Vec<EventModel>) -> Element {
    let mut view_mode: Signal<ViewMode> = use_signal(|| ViewMode::Timeline);
    
    // Вычислить статистику
    let total_time: u64 = events.iter().map(|e| e.duration).sum();
    let total_minutes = total_time / 60;
    let total_hours = total_minutes / 60;
    let remaining_minutes = total_minutes % 60;
    
    let mut unique_apps: Vec<String> = events.iter()
        .map(|e| e.window.process_name.clone())
        .collect();
    unique_apps.sort();
    unique_apps.dedup();

    rsx! {
        div { 
            class: "flex flex-col gap-0 h-full w-full ",
            
            // Верхняя шапка с статистикой
            div {
                 class: "flex flex-row items-center justify-between gap-4 px-2  ",
                
                
                // Правая часть - кнопки переключения вида
                div {
                    class: "flex flex-row items-center gap-2 bg-zinc-900/50 rounded-lg border border-zinc-700/50 p-1",
                    
                    button {
                        class: format!(
                            "px-4 py-2 rounded-md font-semibold transition-all {} {}",
                            if view_mode() == ViewMode::Timeline {
                                "bg-cyan-500/80 text-white shadow-lg shadow-cyan-500/50"
                            } else {
                                "text-gray-400 hover:text-white hover:bg-zinc-800"
                            },
                            if view_mode() == ViewMode::Timeline { "" } else { "opacity-70" }
                        ),
                        onclick: move |_| view_mode.set(ViewMode::Timeline),
                        "📊 График"
                    }
                    
                    button {
                        class: format!(
                            "px-4 py-2 rounded-md font-semibold transition-all {} {}",
                            if view_mode() == ViewMode::List {
                                "bg-purple-500/80 text-white shadow-lg shadow-purple-500/50"
                            } else {
                                "text-gray-400 hover:text-white hover:bg-zinc-800"
                            },
                            if view_mode() == ViewMode::List { "" } else { "opacity-70" }
                        ),
                        onclick: move |_| view_mode.set(ViewMode::List),
                        "📋 Список"
                    }
                    
                    button {
                        class: format!(
                            "px-4 py-2 rounded-md font-semibold transition-all {} {}",
                            if view_mode() == ViewMode::Stats {
                                "bg-green-500/80 text-white shadow-lg shadow-green-500/50"
                            } else {
                                "text-gray-400 hover:text-white hover:bg-zinc-800"
                            },
                            if view_mode() == ViewMode::Stats { "" } else { "opacity-70" }
                        ),
                        onclick: move |_| view_mode.set(ViewMode::Stats),
                        "📈 Статистика"
                    }
                }
            }
            
            // Основной контент
            match view_mode() {
                ViewMode::Timeline => rsx! {
                    div {
                        class: "flex-1 flex flex-row gap-2 p-4",
                        
                        div {
                            class: "flex-1 flex ",
                            
                            EventsTimeline { 
                                events: events.clone(), 
                                orientation: TimelineOrientation::Vertical 
                            }
                        }
                        div {
                            class: "hidden lg:flex w-80 flex-col gap-3 bg-zinc-900/30 rounded-lg border border-zinc-700/30 p-4",
                            
                            // Топ приложений
                            div {
                                class: "flex flex-col gap-3",
                                
                                h3 { 
                                    class: "text-sm font-bold text-cyan-400 uppercase tracking-widest",
                                    "Топ Приложений"
                                }
                                
                                {
                                    let mut app_times: std::collections::HashMap<String, u64> = 
                                        std::collections::HashMap::new();
                                    
                                    for event in &events {
                                        *app_times.entry(event.window.process_name.clone())
                                            .or_insert(0) += event.duration;
                                    }
                                    
                                    let mut app_vec: Vec<_> = app_times.into_iter().collect();
                                    app_vec.sort_by(|a, b| b.1.cmp(&a.1));
                                    
                                    rsx! {
                                        {app_vec.into_iter().take(10).map(|(app, duration)| {
                                            let mins = duration / 60;
                                            let hours = mins / 60;
                                            let remaining = mins % 60;
                                            
                                            rsx! {
                                                div {
                                                    class: "flex flex-row items-center justify-between p-2 bg-zinc-800/40 rounded-md hover:bg-zinc-700/50 transition-colors",
                                                    
                                                    span {
                                                        class: "text-sm font-semibold text-gray-300 truncate",
                                                        "{app}"
                                                    }
                                                    
                                                    span {
                                                        class: "text-sm font-bold text-green-400 whitespace-nowrap ml-2",
                                                        if hours > 0 {
                                                            "{hours}ч {remaining}м"
                                                        } else {
                                                            "{mins}м"
                                                        }
                                                    }
                                                }
                                            }
                                        })}
                                    }
                                }
                            }
                            
                            // Сегодняшняя активность
                            div {
                                class: "flex flex-col gap-3 pt-4 border-t border-zinc-700/50",
                                
                                h3 { 
                                    class: "text-sm font-bold text-purple-400 uppercase tracking-widest",
                                    "Сегодня"
                                }
                                
                                div {
                                    class: "flex flex-col gap-2",
                                    
                                    div {
                                        class: "flex flex-row justify-between items-center",
                                        span { class: "text-xs text-gray-400", "Начало дня" }
                                        span { class: "text-sm font-semibold text-gray-200", "00:00" }
                                    }
                                    
                                    div {
                                        class: "flex flex-row justify-between items-center",
                                        span { class: "text-xs text-gray-400", "Конец дня" }
                                        span { class: "text-sm font-semibold text-gray-200", "23:59" }
                                    }
                                    
                                    div {
                                        class: "h-1 bg-gradient-to-r from-cyan-500 via-purple-500 to-pink-500 rounded-full mt-2"
                                    }
                                }
                            }
                        }
                    }
                },
                ViewMode::List => rsx! {
                    div {
                        class: "flex-1 flex flex-col p-4",
                        EventsList { events: events.clone() }
                    }
                },
                ViewMode::Stats => rsx! {
                    div {
                        class: "flex-1 flex flex-col gap-4 p-4",
                        
                        // Общая статистика
                        div {
                            class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 flex-shrink-0",
                            
                            // Карточка статистики
                            div {
                                class: "bg-gradient-to-br from-cyan-900/30 to-cyan-950/30 border border-cyan-700/50 rounded-lg p-6 hover:from-cyan-900/50 hover:to-cyan-950/50 transition-all",
                                
                                div { class: "text-xs uppercase text-cyan-400 font-bold tracking-wider", "Всего событий" }
                                div { class: "text-4xl font-black text-cyan-400 mt-2", "{events.len()}" }
                            }
                            
                            div {
                                class: "bg-gradient-to-br from-purple-900/30 to-purple-950/30 border border-purple-700/50 rounded-lg p-6 hover:from-purple-900/50 hover:to-purple-950/50 transition-all",
                                
                                div { class: "text-xs uppercase text-purple-400 font-bold tracking-wider", "Приложений" }
                                div { class: "text-4xl font-black text-purple-400 mt-2", "{unique_apps.len()}" }
                            }
                            
                            div {
                                class: "bg-gradient-to-br from-green-900/30 to-green-950/30 border border-green-700/50 rounded-lg p-6 hover:from-green-900/50 hover:to-green-950/50 transition-all",
                                
                                div { class: "text-xs uppercase text-green-400 font-bold tracking-wider", "Часов" }
                                div { class: "text-4xl font-black text-green-400 mt-2", "{total_hours}" }
                            }
                            
                            div {
                                class: "bg-gradient-to-br from-pink-900/30 to-pink-950/30 border border-pink-700/50 rounded-lg p-6 hover:from-pink-900/50 hover:to-pink-950/50 transition-all",
                                
                                div { class: "text-xs uppercase text-pink-400 font-bold tracking-wider", "Минут" }
                                div { class: "text-4xl font-black text-pink-400 mt-2", "{remaining_minutes}" }
                            }
                        }
                        
                        // График по часам
                        div {
                            class: "bg-zinc-900/30 border border-zinc-700/30 rounded-lg p-6",
                            
                            h3 { class: "text-xl font-bold text-cyan-400 mb-4", "📊 Активность по часам" }
                            
                            div {
                                class: "flex flex-col gap-3",
                                
                                {(0..24).map(|hour| {
                                    let empty: Vec<EventModel> = Vec::new();
                                    let hour_events = events.iter()
                                        .filter(|e| {
                                            let dt = convert_ts_to_local_date(e.timestamp);
                                            dt.hour() == hour
                                        })
                                        .count();
                                    
                                    let total_hour_time: u64 = events.iter()
                                        .filter(|e| {
                                            let dt = convert_ts_to_local_date(e.timestamp);
                                            dt.hour() == hour
                                        })
                                        .map(|e| e.duration)
                                        .sum();
                                    
                                    let bar_width = if total_hour_time > 0 {
                                        ((total_hour_time as f32 / 3600.0) * 100.0).min(100.0)
                                    } else {
                                        0.0
                                    };
                                    
                                    rsx! {
                                        div {
                                            class: "flex flex-row items-center gap-3",
                                            
                                            span { 
                                                class: "w-12 text-right font-semibold text-gray-400",
                                                {
                                                    let h = format!("{:02}:00", hour);
                                                    h
                                                }
                                            }
                                            
                                            div {
                                                class: "flex-1 relative h-8 bg-zinc-800/50 rounded-md",
                                                
                                                if bar_width > 0.0 {
                                                    div {
                                                        class: "h-full bg-gradient-to-r from-cyan-500 to-purple-500 rounded-md transition-all",
                                                        style: format!("width: {}%;", bar_width)
                                                    }
                                                }
                                                
                                                span {
                                                    class: "absolute inset-0 flex items-center justify-center text-sm font-bold text-white opacity-80",
                                                    if hour_events > 0 {
                                                        "{hour_events} • {total_hour_time / 60}м"
                                                    } else {
                                                        "—"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                })}
                            }
                        }
                    }
                }
            }
        }
    }
}