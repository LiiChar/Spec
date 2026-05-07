use std::cmp::Ordering;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};
use dioxus::prelude::*;

use crate::{
    core::{EventModel, EventType, JobModel},
    lib::{convert_ts_to_local_date, format_duration_short, get_process_color},
    ui::{TimelineOrientation, Tooltip, TooltipAlign, use_app},
};

#[derive(Props, PartialEq, Clone)]
pub struct EventsElementProps {
    pub events: Vec<EventModel>,
    #[props(default = Signal::new(None))]
    pub selected_job: WriteSignal<Option<JobModel>>,
    #[props(default = vec![])]
    pub jobs: Vec<JobModel>,
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

pub fn sort_by_duration(a: &JobModel, b: &JobModel) -> Ordering {
    (&b.end_ts - &b.start_ts).cmp(&((&a.end_ts - &a.start_ts)))
}

#[component]
pub fn EventElement(props: EventsElementProps) -> Element {
    let app = use_app();
    let mut jobs = props.jobs.clone();

    jobs.sort_by(sort_by_duration);

    let mut selected_job = props.selected_job;

    let is_current_hour = {
        let now = Local::now().hour();
        now >= props.start_hour && now <= props.end_hour
    };

    rsx! {
        div {
            class: format!(
                " w-full h-full relative transition-all duration-200 border border-border/70 {} {} {} {}",
                if props.orientation == TimelineOrientation::Horizontal {
                    "max-w-[calc(100%/24)] max-h-full"
                } else {
                    "max-w-full"
                },
                if !props.events.is_empty() {
                    "bg-foreground/5 hover:bg-foreground/6"
                } else {
                    "bg-foreground/3 hover:bg-foreground/3"
                },
                if is_current_hour { "current-hour" } else { "" },
                props.class,
            ),
            style: match props.orientation {
                TimelineOrientation::Vertical => format!("{}", props.style),
                TimelineOrientation::Horizontal => props.style.clone(),
            },

            {
                jobs
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, job)| {
                        let range_start = props.start_hour * 3600;
                        let range_end = (props.end_hour + 1) * 3600;

                        let (mut start_sec, mut end_sec) = if job.start_ts > 86_400 {
                            let day_start = app
                                .day
                                .read()
                                .date_naive()
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_local_timezone(chrono::Local)
                                .unwrap()
                                .timestamp();

                            (
                                job.start_ts - day_start,
                                job.end_ts - day_start,
                            )
                        } else {
                            (job.start_ts, job.end_ts)
                        };

                        if end_sec < start_sec {
                            end_sec += 86400;
                        }

                        let clamped_start = start_sec.max(range_start as i64);
                        let clamped_end = end_sec.min(range_end as i64);

                        if clamped_end < clamped_start {
                            return None;
                        }

                        let duration_sec = (clamped_end - clamped_start).max(1) as f32;
                        let total_seconds = (range_end - range_start) as f32;
                        let event_start_sec = (clamped_start - range_start as i64) as f32;
                        let offset = (event_start_sec / total_seconds) * 100.0;
                        let size = (duration_sec / total_seconds) * 100.0;
                        let is_select = match props.selected_job.read().clone() {
                            Some(j) => {
                                format!("{}{}{}", job.name, job.start_ts, job.end_ts)
                                    == format!("{}{}{}", j.name, j.start_ts, j.end_ts)
                            }
                            None => false,
                        };
                        let has_prev = start_sec < range_start.into();
                        let has_next = end_sec > range_end.into();
                        
                        Some({
                            let value = job.clone();
                            rsx! {
                                div {
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        selected_job.set(Some(value.clone()));
                                    },
                                    onblur: move |_| {
                                        selected_job.set(None);
                                    },
                                    class: format!(
                                        "absolute group w-1 z-100  cursor-pointer transition-all overflow-hidden {} {}",
                                        has_prev.then(|| "").unwrap_or("rounded-tl-sm"),
                                        has_next.then(|| "").unwrap_or("rounded-bl-sm"),
                                    ),
                                    style: format!(
                                        "{} background-color: {};",
                                        match props.orientation {
                                            TimelineOrientation::Vertical => {
                                                format!(
                                                    "top: {}%; height: {}%; width: {}; right: {}px;",
                                                    offset,
                                                    size.max(0.5),
                                                    if is_select { "3px" } else { "3px" },
                                                    i * 3
                                                )
                                            }
                                            TimelineOrientation::Horizontal => {
                                                format!(
                                                    "left: {}%; width: {}%; height: {}; top: {}px;",
                                                    offset,
                                                    size.max(0.5),
                                                    if is_select { "3px" } else { "3px" },
                                                    i * 3
                                                )
                                            }
                                        },
                                        job.color,
                                    ),
                                    Tooltip {
                                        align: TooltipAlign::Left,
                                        text: "{job.name}",
                                        at_cursor: true,
                                        div {
                                            class: "w-full h-full"
                                        }
                                    }
                                }
                            }
                        })
                })
            }

            {
                props
                    .events
                    .iter()
                    .map(|event| {
                        let start_dt = convert_ts_to_local_date(event.timestamp);
                        let end_dt = start_dt
                            + chrono::Duration::milliseconds(event.duration as i64);
                        let time_range = format!(
                            "{} - {}",
                            start_dt.format("%H:%M:%S"),
                            end_dt.format("%H:%M:%S"),
                        );
                        let duration_formatted = format_duration_short(event.duration);
                        let start_sec = start_dt.minute() * 60 + start_dt.second();
                        let duration_sec = event.duration as f32 / 1000.0;
                        let total_hours = (props.end_hour - props.start_hour + 1) as f32;
                        let total_seconds = total_hours * 3600.0;
                        let event_start_sec = (start_dt.hour() - props.start_hour) as f32
                            * 3600.0 + start_dt.minute() as f32 * 60.0
                            + start_dt.second() as f32;
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
                        }
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
                                    "timeline-event absolute group left-0 right-0 {} cursor-pointer transition-all overflow-visible",
                                    color,
                                ),
                                style: match props.orientation {
                                    TimelineOrientation::Vertical => {
                                        format!("top: {}%; height: {}%; width: 100%;", offset, size.max(0.5))
                                    }
                                    TimelineOrientation::Horizontal => {
                                        format!("left: {}%; width: {}%; height: 100%;", offset, size.max(0.5))
                                    }
                                },
                                Tooltip {
                                    align: TooltipAlign::Top,
                                    at_cursor: true,
                                    target: Some({
                                        rsx! {
                                            div {
                                                class: "p-2 text-xs whitespace-nowrap ",
                                                style: "min-width: 220px; max-width: 320px;",

                                                div { class: "font-bold text-cyan-400", "{process_name}" }
                                                div { class: "text-gray-300 overflow-hidden text-ellipsis", "{window_title}" }
                                                div { class: "text-gray-300", "{time_range}" }
                                                div { class: "text-amber-300", "{duration_formatted}" }
                                            }
                                        }
                                    })
                                }

                                if main_label.is_some() || show_duration {
                                    div { class: "absolute inset-0 flex flex-row gap-2 items-center justify-center text-center pointer-events-none",

                                        if let Some(label) = main_label {
                                            span { class: if size >= 10.0 { "max-w-full truncate whitespace-nowrap text-[10px] font-semibold text-white/90 leading-none" } else { "max-w-full truncate whitespace-nowrap text-[9px] font-medium text-white/85 leading-none" },
                                                "{label}"
                                            }
                                        }

                                        if show_duration {
                                            div { "-" }
                                            span {
                                                class: format!(
                                                    "max-w-full truncate whitespace-nowrap leading-none text-white/80 {}",
                                                    if has_main_label { "mt-0.5 text-[8px]" } else { "text-[9px] font-semibold" },
                                                ),
                                                "{duration_formatted}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    })
            }
        }
    }
}
