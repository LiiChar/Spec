use chrono::{Local, Timelike};
use dioxus::prelude::*;

use crate::{
    core::{EventModel, EventType},
    lib::{convert_ts_to_local_date, format_duration_short, get_process_color},
    ui::TimelineOrientation,
};

#[derive(Props, PartialEq, Clone)]
pub struct EventsElementProps {
    pub events: Vec<EventModel>,
    #[props(default = 0)]
    pub start_hour: u32,
    #[props(default = 0)]
    pub end_hour: u32,
    #[props(default = TimelineOrientation::Vertical)]
    pub orientation: TimelineOrientation,
    #[props(default = "".to_string())]
    pub class: String,
    #[props(default = "".to_string())]
    pub style: String,
}

#[component]
pub fn EventElement(props: EventsElementProps) -> Element {
    let is_current_hour = {
        let now = Local::now().hour();
        now >= props.start_hour && now <= props.end_hour
    };

    rsx! {
        div {
            class: format!(
                "rounded-sm w-full h-full relative transition-all duration-200 border border-border/70 {} {} {} {}",
                if props.orientation == TimelineOrientation::Horizontal {
                    "max-w-[calc(100%/24)] max-h-full"
                } else {
                    "max-w-full"
                },
                if !props.events.is_empty() {
                    "bg-zinc-900/50 hover:bg-zinc-800/70"
                } else {
                    "bg-zinc-900/20 hover:bg-zinc-800/30"
                },
                if is_current_hour { "current-hour" } else { "" },
                props.class
            ),
            style: match props.orientation {
                TimelineOrientation::Vertical => format!("{}", props.style),
                TimelineOrientation::Horizontal => props.style.clone(),
            },

            // span {
            //     class: "absolute left-1.5 top-1 text-[10px] z-1 font-semibold opacity-60 pointer-events-none",
            //     {format!("{:02}:00", props.hour)}
            // }

            {props.events.iter().map(|event| {
                let start_dt = convert_ts_to_local_date(event.timestamp);
                let end_dt = start_dt + chrono::Duration::milliseconds(event.duration as i64);
                let time_range = format!(
                    "{} - {}",
                    start_dt.format("%H:%M:%S"),
                    end_dt.format("%H:%M:%S")
                );
                let duration_formatted = format_duration_short(event.duration);

                let start_sec = start_dt.minute() * 60 + start_dt.second();
                let duration_sec = event.duration as f32 / 1000.0;
                let total_hours = (props.end_hour - props.start_hour + 1) as f32;
                let total_seconds = total_hours * 3600.0;

                let event_start_sec =
                    (start_dt.hour() - props.start_hour) as f32 * 3600.0 +
                    start_dt.minute() as f32 * 60.0 +
                    start_dt.second() as f32;

                let offset = (event_start_sec / total_seconds) * 100.0;
                let size = (duration_sec / total_seconds) * 100.0;

                let window_info = event.window.as_ref();
                let process_name = window_info
                    .map(|window| window.process_name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                let short_process_name: String = process_name.chars().take(10).collect();
                let window_title = window_info
                    .map(|window| window.title.clone())
                    .unwrap_or_else(|| "N/A".to_string());
                let pid_str = window_info
                    .map(|window| window.pid.to_string())
                    .unwrap_or_else(|| "0".to_string());

                let mut color = get_process_color(&process_name).to_owned();

                if matches!(event.event_type, EventType::Idle) {
                    color += "/50";
                };

                let main_label = if size >= 10.0 {
                    Some(process_name.clone())
                } else if size >= 4.0 {
                    Some(short_process_name)
                } else {
                    None
                };
                let has_main_label = main_label.is_some();
                let show_duration = size >= 6.0;

                rsx! {
                    div {
                        key: "{event.timestamp}-{event.duration}-{pid_str}",
                        class: format!(
                            "absolute group left-0 right-0 {} cursor-pointer transition-all overflow-visible",
                            color
                        ),
                        style: match props.orientation {
                            TimelineOrientation::Vertical => format!(
                                "top: {}%; height: {}%; width: 100%;",
                                offset,
                                size.max(0.5)
                            ),
                            TimelineOrientation::Horizontal => format!(
                                "left: {}%; width: {}%; height: 100%;",
                                offset,
                                size.max(0.5)
                            ),
                        },
                        div {
                            class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-1 hidden group-hover:block z-50 bg-background/80 backdrop-blur-lg text-white p-2 rounded-md shadow-lg text-xs whitespace-nowrap border border-zinc-700 pointer-events-none",
                            style: "min-width: 220px; max-width: 320px;",

                            div { class: "font-bold text-cyan-400", "{process_name}" }
                            div { class: "text-gray-300 overflow-hidden text-ellipsis", "{window_title}" }
                            div { class: "text-gray-300", "{time_range}" }
                            div { class: "text-amber-300", "{duration_formatted}" }
                        }

                        if main_label.is_some() || show_duration {
                            div {
                                class: "absolute inset-0 flex flex-row gap-2 items-center justify-center text-center pointer-events-none",

                                if let Some(label) = main_label {
                                    span {
                                        class: if size >= 10.0 {
                                            "max-w-full truncate whitespace-nowrap text-[10px] font-semibold text-white/90 leading-none"
                                        } else {
                                            "max-w-full truncate whitespace-nowrap text-[9px] font-medium text-white/85 leading-none"
                                        },
                                        "{label}"
                                    }
                                }

                                if show_duration {
                                    div {
                                        "-"
                                    }
                                    span {
                                        class: format!(
                                            "max-w-full truncate whitespace-nowrap leading-none text-white/80 {}",
                                            if has_main_label { "mt-0.5 text-[8px]" } else { "text-[9px] font-semibold" }
                                        ),
                                        "{duration_formatted}"
                                    }
                                }
                            }
                        }
                    }
                }
            })}
        }
    }
}
