use std::cmp::Ordering;

use chrono::{Local, Timelike};
use dioxus::prelude::*;
use palette::{Srgba, WithAlpha};

use crate::{
    core::{EventModel, EventType, JobModel, TagModel, with_database},
    lib::{CronExpr, color::{foreground_color, icon_bg_color, idle_color, set_alpha, soften_color}, convert_ts_to_local_date, format_duration_short, get_process_color}, ui::{components::{events::TimelineOrientation, ui::tooltip::{Tooltip, TooltipAlign}}, context::use_settings},
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
    #[props(default = false)]
    pub is_selected: bool,
}

pub fn sort_by_duration(a: &JobModel, b: &JobModel) -> Ordering {
    (b.end_ts - b.start_ts).cmp(&(a.end_ts - a.start_ts))
}

fn event_range_in_segment(
    event: &EventModel,
    start_hour: u32,
    end_hour: u32,
) -> (f32, f32) {
    let start_dt = convert_ts_to_local_date(event.timestamp);

    let duration_sec = event.duration as f32 / 1000.0;
    let total_hours = (end_hour - start_hour + 1) as f32;
    let total_seconds = total_hours * 3600.0;

    let event_start_sec =
        (start_dt.hour() - start_hour) as f32 * 3600.0
            + start_dt.minute() as f32 * 60.0
            + start_dt.second() as f32;

    let offset = (event_start_sec / total_seconds) * 100.0;
    let size = (duration_sec / total_seconds) * 100.0;

    (offset, size.max(0.05))
}

fn overlaps_percent(
    a_start: f32,
    a_size: f32,
    b_start: f32,
    b_size: f32,
) -> bool {
    let a_end = a_start + a_size;
    let b_end = b_start + b_size;

    a_start < b_end && b_start < a_end
}

fn compute_event_lanes(
    events: &[EventModel],
    start_hour: u32,
    end_hour: u32,
) -> Vec<usize> {
    if events.is_empty() {
        return Vec::new();
    }

    let mut lanes: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut result = Vec::with_capacity(events.len());

    for event in events {
        let (offset, size) = event_range_in_segment(event, start_hour, end_hour);

        let mut placed = false;

        for (lane_index, lane) in lanes.iter_mut().enumerate() {
            let intersects = lane.iter().any(|(other_offset, other_size)| {
                overlaps_percent(offset, size, *other_offset, *other_size)
            });

            if !intersects {
                lane.push((offset, size));
                result.push(lane_index);
                placed = true;
                break;
            }
        }

        if !placed {
            lanes.push(vec![(offset, size)]);
            result.push(lanes.len() - 1);
        }
    }

    result
}

#[component]
pub fn EventElement(props: EventsElementProps) -> Element {
    let settings = use_settings();

    let mut jobs = props.jobs;
    let orientation = props.orientation;

    jobs.sort_by(sort_by_duration);

    let mut selected_job = props.selected_job;
    let has_events = !props.events.is_empty();

    let is_current_hour = {
        let now = Local::now().hour();
        now >= props.start_hour && now <= props.end_hour
    };

    let count_job = use_signal(|| jobs.len());

    rsx! {
        div {
            class: format!(
                " w-full h-full relative transition-all duration-200 border border-border/70 {} {} {} {}",
                if orientation == TimelineOrientation::Horizontal {
                    "max-w-[calc(100%/24)] max-h-full"
                } else {
                    "max-w-full"
                },
                if has_events {
                    "bg-foreground/5 hover:bg-foreground/6"
                } else {
                    "bg-foreground/3 hover:bg-foreground/3"
                },
                if is_current_hour { "current-hour" } else { "" },
                props.class,
            ),
            style: props.style.clone(),

            {
                let jobs_ref = &jobs;
                jobs
                    .iter()
                    .enumerate()
                    .filter_map(|(i, job)| {

                        let start_ts = job.start_ts;
                        let end_ts = job.end_ts;

                        let range_start = (props.start_hour * 3_600_000) as i64;
                        let range_end = ((props.end_hour + 1) * 3_600_000) as i64;

                        let (start_ms, mut end_ms) = {
                            let default_start = convert_ts_to_local_date(start_ts as u64);
                            let default_end = convert_ts_to_local_date(end_ts as u64);

                            let cron = job.cron.as_ref()
                                .and_then(|c| CronExpr::parse(c).ok());

                            let (h_start, h_end, m_start, m_end, s_start, s_end) = if let Some(cron) = cron {
                                let c_h = cron.hour;
                                let c_m = cron.minute;
                                let c_s = cron.second;

                                let parse_or_range = |field: crate::lib::Field, def: u32| match field {
                                    crate::lib::Field::Range(s, e) => (s, e),
                                    _ => (def, def),
                                };

                                let (hs, he) = parse_or_range(c_h, default_start.hour() as u32);
                                let (ms, me) = parse_or_range(c_m, default_start.minute() as u32);
                                let (ss, se) = parse_or_range(c_s, default_start.second() as u32);

                                (hs, he, ms, me, ss, se)
                            } else {
                                (
                                    default_start.hour() as u32,
                                    default_end.hour() as u32,
                                    default_start.minute() as u32,
                                    default_end.minute() as u32,
                                    default_start.second() as u32,
                                    default_end.second() as u32,
                                )
                            };

                            let build_ms = |h: u32, m: u32, s: u32| {
                                ((h as i64) * 3600 + (m as i64) * 60 + (s as i64)) * 1000
                            };

                            let start = build_ms(h_start, m_start, s_start);
                            let end = build_ms(h_end, m_end, s_end);

                            (start, end)
                        };

                        if end_ms < start_ms {
                            end_ms += 86_400_000;
                        }

                        let clamped_start = start_ms.max(range_start);
                        let clamped_end = end_ms.min(range_end);

                        if clamped_end < clamped_start {
                            return None;
                        }

                        let duration_ms = (clamped_end - clamped_start).max(1) as f32;
                        let total_ms = (range_end - range_start) as f32;
                        let event_start_ms = (clamped_start - range_start) as f32;

                        let offset = (event_start_ms / total_ms) * 100.0;
                        let size = (duration_ms / total_ms) * 100.0;

                        let has_prev = start_ms < range_start;
                        let has_next = end_ms > range_end;

                        let job_width = 3;

                        let cron = job.cron.as_ref()
                                .and_then(|c| CronExpr::parse(c).ok()).unwrap();
                                                    

                        let mut border_offset = 0;

                        for index in 0..i {
                            let i_job = &jobs_ref[index];
                            let i_job_cron = i_job.cron.as_ref()
                                .and_then(|c| CronExpr::parse(c).ok()).unwrap();

                            if cron.compare(&i_job_cron, false, Some("+ + + - - - -".to_string())) {
                                border_offset += job_width;
                            }
                        }

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
                                        "absolute group w-1 z-100 cursor-pointer transition-all overflow-hidden {} {}",
                                        if has_prev { "" } else { "rounded-tl-sm" },
                                        if has_next { "" } else { "rounded-bl-sm" },
                                    ),
                                    style: format!(
                                        "{} background-color: {};",
                                        match orientation {
                                            TimelineOrientation::Vertical => {
                                                format!(
                                                    "top: {}%; height: {}%; width: {}px; right: {}px;",
                                                    offset,
                                                    size.max(0.5),
                                                    job_width,
                                                    border_offset
                                                )
                                            }
                                            TimelineOrientation::Horizontal => {
                                                format!(
                                                    "left: {}%; width: {}%; height: {}px; top: {}px;",
                                                    offset,
                                                    size.max(0.5),
                                                    job_width,
                                                    border_offset
                                                )
                                            }
                                        },
                                        job.color,
                                    ),
                                    Tooltip {
                                        align: TooltipAlign::Left,
                                        text: "{job.name}",
                                        at_cursor: true,
                                        gap: 2,
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
                let mut events = props.events;

                

                // длинные сначала
                events.sort_by(|a, b| b.duration.cmp(&a.duration));

                let lanes = compute_event_lanes(
                    &events,
                    props.start_hour,
                    props.end_hour,
                );

                let lane_count = lanes.iter().copied().max().unwrap_or(0) + 1;

                events
                    .into_iter()
                    .enumerate()
                    .map(move |(index, event)| {
                        let lanes_c = lanes.clone();
                        let lane_count_c = lane_count;

                        let start_dt = convert_ts_to_local_date(event.timestamp);
                        let end_dt =
                            start_dt + chrono::Duration::milliseconds(event.duration as i64);

                        let time_range = format!(
                            "{} - {}",
                            start_dt.format("%H:%M:%S"),
                            end_dt.format("%H:%M:%S"),
                        );

                        let duration_formatted = format_duration_short(event.duration);

                        let duration_sec = event.duration as f32 / 1000.0;
                        let total_hours =
                            (props.end_hour - props.start_hour + 1) as f32;
                        let total_seconds = total_hours * 3600.0;

                        let event_start_sec =
                            (start_dt.hour() - props.start_hour) as f32 * 3600.0
                                + start_dt.minute() as f32 * 60.0
                                + start_dt.second() as f32;

                        let offset = (event_start_sec / total_seconds) * 100.0;
                        let size = (duration_sec / total_seconds) * 100.0;

                        let window_info = event.window.as_ref();

                        let process_name = window_info
                            .map(|w| w.process_name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        let short_process_name: String =
                            process_name.chars().take(10).collect();

                        let window_title = window_info
                            .map(|w| w.title.clone())
                            .unwrap_or_else(|| "N/A".to_string());

                        let is_idle = event.event_type == EventType::Idle;
                        let color = window_info
                            .map(|w| w.color.clone() ).unwrap_or_else(|| "rgba(0,0,0,1)".to_string());

                        let mut evt_color = soften_color(&color, 0.4);

                        if !settings.settings.read().soft_event {
                            evt_color = color.clone();
                        }

                        if is_idle {
                            evt_color = idle_color(&evt_color);
                        }

                        let icon_bg_color = icon_bg_color(&evt_color);

                        let track_px = match props.orientation {
                            TimelineOrientation::Vertical => {
                                if props.is_selected { settings.settings.read().selected_segment_height as f32 } else { settings.settings.read().segment_height as f32 }
                            }
                            TimelineOrientation::Horizontal => {
                                if props.is_selected { settings.settings.read().selected_segment_height as f32 } else { settings.settings.read().segment_height as f32 }
                            }
                        };

                        let event_px = (size / 100.0) * track_px;

                        const DOT_THRESHOLD: f32 = 3.0;
                        const DOT_THRESHOLD_ICON: f32 = 30.0;
                        const SHORT_LABEL_THRESHOLD: f32 = 10.0;
                        const FULL_LABEL_THRESHOLD: f32 = 18.0;
                        const DURATION_THRESHOLD: f32 = 26.0;

                        let is_micro = event_px < DOT_THRESHOLD;
                        let is_micro_icon = event_px >= DOT_THRESHOLD_ICON;

                        let label = if event_px >= FULL_LABEL_THRESHOLD {
                            Some(process_name.clone())
                        } else if event_px >= SHORT_LABEL_THRESHOLD {
                            Some(short_process_name)
                        } else {
                            None
                        };

                        let lane = lanes_c[index];


                        rsx! {
                            div {
                                key: "{event.timestamp}",

                                class: format!(
                                    "timeline-event left-1 right-1 absolute group cursor-pointer transition-all overflow-visible rounded-[2px] z-4",
                                    
                                ),

                                style: format!("{} {} {}",match props.orientation {
                                    TimelineOrientation::Vertical => {
                                        if is_micro {
                                            format!(
                                                "top:{}%; height:2px;",
                                                offset,
                                            )
                                        } else {
                                            format!(
                                                "top:{}%; height:{}%;",
                                                offset,
                                                size.max(0.7),
                                            )
                                        }
                                    }
                                    TimelineOrientation::Horizontal => {
                                        if is_micro {
                                            format!(
                                                "left:{}%; width:2px;",
                                                offset,
                                            )
                                        } else {
                                            format!(
                                                "left:{}%; width:{}%;",
                                                offset,
                                                size.max(0.7),
                                            )
                                        }
                                    }
                                }, 
                                    format!("left: {}px; right: {}px;", 
                                        if is_current_hour  {
                                            6
                                        } else {
                                            3
                                        },
                                        (count_job() + 1) * 3
                                    ),
                                    format!("background-color: {};", evt_color)
                                ),

                                Tooltip {
                                    align: TooltipAlign::Top,
                                    at_cursor: true,
                                    target: Some({
                                        rsx! {
                                            div {
                                                class: "p-2 whitespace-nowrap -ml-1 min-w-[220px]   z-100",

                                                div {
                                                    class: "absolute w-[7px] h-[7px] top-1 right-1 rounded-full",
                                                    style: format!("background-color: {};", 
                                                        if is_idle {
                                                            "rgb(91, 98, 108)"
                                                        } else {
                                                            "var(--primary)"
                                                        }
                                                    ),
                                                }

                                                div {
                                                    class: "absolute w-[3px] h-[calc(100%-8px)]  left-1 top-1 bottom-1 rounded-md",
                                                    style: format!("background-color: {};", color),

                                                }

                                                div {
                                                    class: "flex gap-2 items-center overflow-hidden text-ellipsis",
                                                    if let Some(window) = event.window.clone() {
                                                        if let Some(icon) = window.icon_base64 {
                                                            img {
                                                                class: "w-5 h-5 rounded",
                                                                src: icon
                                                            }
                                                        }
                                                    }
                                                    div {
                                                        class: "font-bold text-base text-primary",
                                                        "{window_title}"
                                                    }
                                                }

                                                div {
                                                    class: "text-muted-foreground/60 text-xs text-foreground/55 overflow-hidden text-ellipsis flex gap-1",
                                                    span { "{process_name}" }
                                                }

                                                div {
                                                    class: "text-md",
                                                    "{duration_formatted} ({time_range})"
                                                }
                                            }
                                        }
                                    })
                                }

                                

                                if !is_micro && lane == 0 && label.is_some() {
                                    div {
                                        div {
                                            class: format!("text-[10px] {}", foreground_color(&color, "text-background".to_owned(), "text-foreground".to_owned())),
                                            if settings.settings.read().show_tags {
                                                if settings.settings.read().type_tags != "сircle" {
                                                    div {
                                                        class: "absolute left-10 top-1/2 -translate-y-1/2 h-full py-1 flex items-center justify-center",
                                                        {
                                                            let tags = event.window.as_ref().map(|w| w.tags.clone()).unwrap_or_default();
                                                            rsx! {
                                                                div { 
                                                                    class: "flex gap-0.5 h-full relative z-1",
                                                                    for tag in tags {
                                                                        Tooltip {
                                                                            align: TooltipAlign::Right,
                                                                            text: "{tag.name}",
                                                                            gap: 2,
                                                                            div {
                                                                                class: "w-[5px] h-full rounded-md",
                                                                                style: format!("background-color: {}; height: {}", tag.color, if props.is_selected { "100%" } else { "0px" }),
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
 
                                                   div {
                                                        class: "absolute left-10 top-1/2 -translate-y-1/2 h-full py-1 flex items-center justify-center",
                                                        {
                                                            let tags = event.window.as_ref().map(|w| w.tags.clone()).unwrap_or_default();
                                                            rsx! {
                                                                div { 
                                                                    class: "flex gap-0.5 relative z-1 justify-center items-center",
                                                                    for tag in tags {
                                                                        Tooltip {
                                                                            align: TooltipAlign::Right,
                                                                            text: "{tag.name}",
                                                                            div {
                                                                                class: "w-[6px] rounded-full",
                                                                                style: format!("background-color: {}; height: {}", tag.color, if props.is_selected { "6px" } else { "0px" }),
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                            }

                                            {
                                                let display_mode = &settings.settings.read().type_label;


                                                let show_icon = matches!(display_mode.as_str(), "full");
                                                let show_title = matches!(display_mode.as_str(), "full" | "title");
                                                let show_label = matches!(display_mode.as_str(), "full" | "text" | "label");

                                                rsx! {   
                                                    if let Some(label) = label {
                                                        div {
                                                            class: "absolute left-1/2 -translate-x-1/2 top-1/2 -translate-y-1/2 flex gap-1 items-center justify-center h-full pointer-events-none truncate whitespace-nowrap font-medium leading-none select-none",

                                                            div {
                                                                class: "flex gap-1 justify-center items-center",

                                                                if show_icon && is_micro_icon {
                                                                    if let Some(window) = event.window.clone() {
                                                                        if let Some(icon) = window.icon_base64 {
                                                                            div {
                                                                                class: "rounded-lg p-1",
                                                                                style: format!("background-color: {};", icon_bg_color),
                                                                                img {
                                                                                    class: "w-5 h-5 rounded",
                                                                                    src: icon
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }


                                                                div {
                                                                    class: "flex flex-col",

                                                                    if show_title && (is_micro_icon || !show_label) {
                                                                        div {
                                                                            class: "text-sm font-bold",
                                                                            "{window_title}"
                                                                        }
                                                                    }

                                                                    if show_label && (is_micro_icon || !show_title || show_label) {
                                                                        div {
                                                                            class: if show_title {
                                                                                "opacity-70 -mt-1"
                                                                            } else {
                                                                                ""
                                                                            },
                                                                            "{label}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            div {
                                                class: "whitespace-nowrap absolute right-4 top-1/2 -translate-y-1/2 flex items-center justify-center pointer-events-none select-none",
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
