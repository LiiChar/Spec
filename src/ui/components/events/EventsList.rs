use dioxus::prelude::*;
use chrono::Timelike;

use crate::{core::EventModel, lib::{convert_ts_to_local_date, get_process_color_gradient}};

#[derive(Props, PartialEq, Clone)]
pub struct EventsListProps {
    events: Vec<EventModel>
}

#[component]
pub fn EventsList(props: EventsListProps) -> Element {
    let total_duration: u64 = props.events.iter().map(|e| e.duration).sum();
    let total_minutes = total_duration / 60;
    let total_hours = total_minutes / 60;
    let remaining_mins = total_minutes % 60;

    rsx! {
        div {
            class: "flex flex-col gap-4 w-full h-full",
            
            // Статистика
            div {
                class: "grid grid-cols-3 gap-4 px-4 py-3 bg-zinc-900/50 rounded-lg border border-zinc-700/50 flex-shrink-0",
                
                div {
                    class: "flex flex-col items-center",
                    span { class: "text-gray-400 text-sm", "События" }
                    span { class: "text-2xl font-bold text-cyan-400", "{props.events.len()}" }
                }
                
                div {
                    class: "flex flex-col items-center",
                    span { class: "text-gray-400 text-sm", "Приложения" }
                    span { 
                        class: "text-2xl font-bold text-purple-400",
                        {
                            let mut apps: Vec<String> = props.events.iter()
                                .map(|e| e.window.process_name.clone())
                                .collect();
                            apps.sort();
                            apps.dedup();
                            format!("{}", apps.len())
                        }
                    }
                }
                
                div {
                    class: "flex flex-col items-center",
                    span { class: "text-gray-400 text-sm", "Время" }
                    span { class: "text-2xl font-bold text-green-400", "{total_hours}ч {remaining_mins}м" }
                }
            }
            
            // Список событий (БЕЗ СКРОЛЛА)
            div {
                class: "flex flex-col gap-2 flex-1",
                
                {props.events.iter().rev().take(20).map(|event| {
                    let start_dt = convert_ts_to_local_date(event.timestamp);
                    let end_dt = convert_ts_to_local_date(event.timestamp + event.duration);
                    
                    let start_str = start_dt.format("%H:%M:%S").to_string();
                    let end_str = end_dt.format("%H:%M:%S").to_string();
                    let duration_minutes = event.duration / 60;
                    let color_gradient = get_process_color_gradient(&event.window.process_name);
                    let event_type_str = format!("{:?}", event.event_type);
                    
                    // Подготавливаем данные для drag & drop
                    let process_name = event.window.process_name.clone();
                    let pid_str = format!("{}", event.window.pid);

                    rsx! {
                        div {
                            class: "group relative flex flex-row items-center gap-3 px-4 py-3 bg-zinc-900/40 hover:bg-zinc-800/60 rounded-lg border border-zinc-700/30 hover:border-zinc-600/50 transition-all cursor-pointer",
                            
                            // Цветной индикатор слева (с drag & drop)
                            div {
                                class: format!("flex items-center justify-center w-10 h-10 rounded-lg bg-gradient-to-br {} text-white font-bold text-lg select-none cursor-grab active:cursor-grabbing hover:scale-110 transition-transform", color_gradient),
                                draggable: "true",
                                ondragstart: move |evt| {
                                    // Передаем данные о приложении при перетаскивании
                                    let data = evt.data_transfer();
                                    let _ = data.set_data("text/plain", &process_name);
                                    let _ = data.set_data("application/x-process-info", &pid_str);
                                },
                            }
                            
                            // Основная информация
                            div {
                                class: "flex-1 flex flex-col gap-1",
                                
                                // Название приложения
                                div {
                                    class: "font-semibold text-white text-sm",
                                    "{event.window.process_name}"
                                }
                                
                                // Заголовок окна
                                div {
                                    class: "text-gray-400 text-xs truncate max-w-[300px]",
                                    title: "{event.window.title}",
                                    "{event.window.title}"
                                }
                                
                                // Время
                                div {
                                    class: "text-gray-500 text-[11px]",
                                    "{start_str} → {end_str}"
                                }
                            }
                            
                            // Продолжительность
                            div {
                                class: "flex flex-col items-end gap-1",
                                
                                div {
                                    class: "font-bold text-cyan-400 text-sm",
                                    "{duration_minutes}м"
                                }
                                
                                div {
                                    class: "text-gray-500 text-[10px]",
                                    "{event.duration}с"
                                }
                            }
                            
                            // Подробный тултип при наведении
                            div {
                                class: "absolute top-full left-4 mt-1 hidden group-hover:block z-50 bg-zinc-950 text-white p-3 rounded-md shadow-xl text-xs border border-zinc-700 whitespace-nowrap",
                                
                                div { class: "font-bold text-cyan-400 mb-2", "Детали события" }
                                div { class: "flex flex-col gap-1 text-gray-300",
                                    div { "Процесс: {event.window.process_name}" }
                                    div { "PID: {event.window.pid}" }
                                    div { "Размер окна: {event.window.rect.width}x{event.window.rect.height}" }
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