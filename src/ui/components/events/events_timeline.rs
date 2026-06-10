use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, Timelike};
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdArrowUpToLine, LdBarChart, LdX, LdArrowDownToLine };
use dioxus_free_icons::Icon;
use tokio::time::{Duration as TokioDuration};

use crate::{
    core::{EventModel, EventType, JobModel},
    lib::{CronExpr, convert_ts_to_local_date, merge_events},
    ui::{EventElement, job_modal::JobModal, use_settings, stats_modal::StatsModal},
};

#[derive(PartialEq, Clone, Copy)]
pub enum TimelineOrientation {
    Horizontal,
    Vertical,
}

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    events: ReadSignal<Vec<EventModel>>,
    jobs: ReadSignal<Vec<JobModel>>,
    day: ReadSignal<DateTime<Local>>,
    orientation: TimelineOrientation,
}

fn group_by_hours(events: &[EventModel]) -> HashMap<u32, HashMap<u32, Vec<EventModel>>> {
    let mut result: HashMap<u32, HashMap<u32, Vec<EventModel>>> = HashMap::new();

    for event in events {
        let mut current = convert_ts_to_local_date(event.timestamp);
        let end = convert_ts_to_local_date(event.timestamp + event.duration);

        while current < end {
            let next_hour = current
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                + chrono::Duration::hours(1);

            let slice_end = if next_hour < end { next_hour } else { end };
            let duration = (slice_end.timestamp_millis() - current.timestamp_millis()) as u64;

            result
                .entry(current.day())
                .or_default()
                .entry(current.hour())
                .or_default()
                .push(EventModel {
                    window: event.window.clone(),
                    event_type: event.event_type.clone(),
                    timestamp: current.timestamp_millis() as u64,
                    duration,
                });

            current = slice_end;
        }
    }
    result
}

#[derive(Clone, PartialEq, Debug)]
struct Segment {
    day: DateTime<Local>,
    start: u32,
    end: u32,
    group: Vec<EventModel>,
    has_events: bool,
}

fn group_by_segments(events: &[EventModel]) -> Vec<Segment> {
    if events.is_empty() {
        return Vec::new();
    }

    // --- 1. разбиваем по (день, час) ---
    let mut map: HashMap<(i64, u32), Vec<EventModel>> = HashMap::new();

    for event in events {
        let mut current = convert_ts_to_local_date(event.timestamp);
        let end = convert_ts_to_local_date(event.timestamp + event.duration);

        while current < end {
            let next_hour = current
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                + Duration::hours(1);

            let slice_end = if next_hour < end { next_hour } else { end };
            let duration = (slice_end.timestamp_millis() - current.timestamp_millis()) as u64;

            let day_ts = current
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp_millis();

            map.entry((day_ts, current.hour()))
                .or_default()
                .push(EventModel {
                    window: event.window.clone(),
                    event_type: event.event_type.clone(),
                    timestamp: current.timestamp_millis() as u64,
                    duration,
                });

            current = slice_end;
        }
    }

    // --- 2. нормализуем в timeline (с пустыми часами!) ---
    let mut timeline: Vec<(i64, u32, Vec<EventModel>)> = Vec::new();

    let mut days: Vec<i64> = map.keys().map(|(d, _)| *d).collect();
    days.sort();
    days.dedup();

    for day in days {
        for hour in 0..24 {
            let events = map.remove(&(day, hour)).unwrap_or_default();
            timeline.push((day, hour, events));
        }
    }

    timeline.sort_by_key(|(day, hour, _)| (*day, *hour));

    // --- 3. сегментация ---
    let mut segments = Vec::new();

    let (mut current_day, mut current_start, mut current_events) = {
        let (d, h, e) = timeline[0].clone();
        (d, h, e)
    };

    let mut current_end = current_start;
    let mut current_has_events = !current_events.is_empty();

    for (day, hour, events) in timeline.into_iter().skip(1) {
        let has_events = !events.is_empty();

        let is_same = day == current_day
            && hour == current_end + 1
            && (
                // объединяем только если ОБА пустые
                (!has_events && !current_has_events)
            );

        if is_same {
            current_end = hour;
            current_events.extend(events);
        } else {
            segments.push(Segment {
                day: convert_ts_to_local_date(current_day as u64),
                start: current_start,
                end: current_end,
                group: current_events.clone(),
                has_events: current_has_events,
            });

            current_day = day;
            current_start = hour;
            current_end = hour;
            current_events = events;
            current_has_events = has_events;
        }
    }

    // последний сегмент
    segments.push(Segment {
        day: convert_ts_to_local_date(current_day as u64),
        start: current_start,
        end: current_end,
        group: current_events,
        has_events: current_has_events,
    });

    segments
}


fn y_to_timestamp(
    y: f64,
    segments: &[Segment],
    selected_hour: Option<u32>,
    day_start: u64,
    base: u32,
    expanded: u32,
) -> u64 {
    let mut acc: f64 = 0.0;

    for seg in segments {
        let size = if seg.has_events {
            match selected_hour {
                Some(hour)
                    if hour >= seg.start && hour <= seg.end =>
                {
                    expanded as f64
                }
                _ => base as f64,
            }
        } else {
            base as f64
        };

        if y < acc + size {
            let progress = (y - acc) / size;

            let start_ts =
                day_start + seg.start as u64 * 3_600_000;

            let end_ts =
                day_start + (seg.end as u64 + 1) * 3_600_000;

            let segment_duration = end_ts - start_ts;

            return start_ts
                + (progress * segment_duration as f64) as u64;
        }

        acc += size;
    }

    day_start + 86_400_000
}

fn timestamp_to_y(
    ts: u64,
    segments: &[Segment],
    selected_hour: Option<u32>,
    day_start: u64,
    base: u32,
    expanded: u32,
) -> f64 {
    let mut y = 0.0;

    for seg in segments {
        let size = if seg.has_events {
            match selected_hour {
                Some(hour)
                    if hour >= seg.start && hour <= seg.end =>
                {
                    expanded as f64
                }
                _ => base as f64,
            }
        } else {
            base as f64
        };

        let start_ts =
            day_start + seg.start as u64 * 3_600_000;

        let end_ts =
            day_start + (seg.end as u64 + 1) * 3_600_000;

        if ts >= start_ts && ts < end_ts {
            let progress =
                (ts - start_ts) as f64 / (end_ts - start_ts) as f64;

            return y + progress * size;
        }

        y += size;
    }

    y
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DragState {
    start_y: f64,
    rect_top: f64,
}

#[component]
pub fn EventsTimelineView(props: EventsCalendarProps) -> Element {
    let settings = use_settings();

    let mut selected_hour = use_signal(|| None::<u32>);
    let mut selected_job = use_signal(|| None::<JobModel>);

    let day_events = use_memo(move || {
        let show_idle = settings.settings.read().show_idle_events;

        let merged = {
            let items = props.events.read();

            let items = items
                .iter()
                .filter_map(|i| {
                    if !show_idle && i.event_type == EventType::Idle {
                        None
                    } else {
                        Some(i.clone())
                    }
                })
                .collect::<Vec<_>>();

            let merged = merge_events(items);

            let px_per_hour = selected_hour()
                .map(|_| 800.0)
                .unwrap_or(80.0);

            // merge_visual_density(merged, px_per_hour, 3.0)
            merged
        };

        let segments = group_by_segments(&merged);
        let today = props.day.read();

        segments
            .into_iter()
            .filter(|segment| {
                today.format("%h%d").to_string() == segment.day.format("%h%d").to_string()
            })
            .collect::<Vec<_>>()
    });

    let mut node_element = use_signal(|| None);

    let mut drag_state = use_signal(|| None::<DragState>);
    let mut moved = use_signal(|| false);

    let mut selected_start_position =
        use_signal(|| None::<f64>);

    let mut selected_end_position =
        use_signal(|| None::<f64>);

    let mut selected_events = use_signal(|| Vec::new());
    
    let mut current_time = use_signal(|| chrono::Local::now().timestamp_millis());

    use_effect(move || {
        if !settings.settings.read().show_current_time_line {
            return;
        }
        spawn(async move {
            loop {
                current_time.set(chrono::Local::now().timestamp_millis());
                tokio::time::sleep(TokioDuration::from_millis(1000)).await;
            }
        });
    });

    let mut visible_stats = use_signal(|| false);

    fn start_selection(
        y: f64,
        rect_top: f64,
        drag_state: &mut Signal<Option<DragState>>,
        moved: &mut Signal<bool>,
        start: &mut Signal<Option<f64>>,
        end: &mut Signal<Option<f64>>,
    ) {
        drag_state.set(Some(DragState {
            start_y: y,
            rect_top,
        }));

        moved.set(false);

        start.set(Some(y));
        end.set(Some(y));
    }

    fn update_selection(
        mouse_y: f64,
        drag_state: DragState,
        moved: &mut Signal<bool>,
        start: &mut Signal<Option<f64>>,
        end: &mut Signal<Option<f64>>,
    ) {
        let y = mouse_y - drag_state.rect_top;

        if (y - drag_state.start_y).abs() > 5.0 {
            moved.set(true);
        }

        start.set(Some(
            drag_state.start_y.min(y)
        ));

        end.set(Some(
            drag_state.start_y.max(y)
        ));
    }

    fn finish_selection(
        drag_state: &mut Signal<Option<DragState>>,
    ) {
        drag_state.set(None);
    }


    rsx! {
        StatsModal {
            visible: visible_stats,
            on_close: move |_| {
                visible_stats.set(false);
                selected_start_position.set(None);
                selected_end_position.set(None);
            },
            events: selected_events().clone(),
            start_ts: y_to_timestamp(selected_start_position().unwrap_or(0.0), &day_events.read(), selected_hour(), props
                    .day
                    .read()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp_millis() as u64,
                    settings.settings.read().segment_height,
                    settings.settings.read().selected_segment_height,
                ) as i64,
            end_ts: y_to_timestamp(selected_end_position().unwrap_or(0.0), &day_events.read(), selected_hour(), props
                    .day
                    .read()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp_millis() as u64,
                    settings.settings.read().segment_height,
                    settings.settings.read().selected_segment_height) as i64,
        }
        JobModal {
            job: selected_job.read().clone(),
            on_close: move |_| selected_job.set(None),
        }
        div {
            class: format!(
                "flex w-full h-min relative rounded-md overflow-hidden border border-border/50 bg-card/30 user-select-none {}",
                match props.orientation {
                    TimelineOrientation::Horizontal => "flex-row",
                    TimelineOrientation::Vertical => "flex-col",
                },
            ),
            onmounted: move |cx| {
                node_element.set(Some(cx.data()));
            },

            onpointerdown: move |evt| {
                evt.stop_propagation();
                let node = node_element.read().clone();

                spawn(async move {
                    if let Some(node) = node {
                        if let Ok(rect) = node.get_client_rect().await {
                            let y =
                                evt.client_coordinates().y as f64
                                - rect.origin.y;

                            start_selection(
                                y,
                                rect.origin.y,
                                &mut drag_state,
                                &mut moved,
                                &mut selected_start_position,
                                &mut selected_end_position,
                            );
                        }
                    }
                });
            },

            onpointermove: move |evt| {
                let Some(state) = *drag_state.read() else {
                    return;
                };

                update_selection(
                    evt.client_coordinates().y as f64,
                    state,
                    &mut moved,
                    &mut selected_start_position,
                    &mut selected_end_position,
                );
            },

            onpointerup: move |_| {
                finish_selection(&mut drag_state);

                let Some(start_y) = *selected_start_position.read() else {
                    return;
                };

                let Some(end_y) = *selected_end_position.read() else {
                    return;
                };

                let day_start = props
                    .day
                    .read()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp_millis() as u64;

                let start_ts =
                    y_to_timestamp(start_y, &day_events.read(), selected_hour(), day_start,
                    settings.settings.read().segment_height,
                    settings.settings.read().selected_segment_height);

                let end_ts =
                    y_to_timestamp(end_y, &day_events.read(), selected_hour(), day_start,
                    settings.settings.read().segment_height,
                    settings.settings.read().selected_segment_height);

                let selection_start = start_ts.min(end_ts);
                let selection_end = start_ts.max(end_ts);

                let mut s_events = Vec::new();

                for event in props.events.read().iter() {
                    let event_start = event.timestamp;
                    let event_end = event.timestamp + event.duration;

                    let intersects =
                        event_start < selection_end &&
                        event_end > selection_start;

                    if intersects {
                        s_events.push(event.clone());
                    }
                }

                s_events.sort_by_key(|e| e.timestamp);

                selected_events.set(s_events.clone());

                if selected_start_position().unwrap_or(0.0) == selected_end_position().unwrap_or(0.0) {
                    selected_start_position.set(None);
                    selected_end_position.set(None);
                }

            },
            {
                let start = selected_start_position().unwrap_or(0.0);
                let end = selected_end_position().unwrap_or(0.0);

                let day_start = props
                    .day
                    .read()
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp_millis() as u64;

                let start_ts =
                    y_to_timestamp(start, &day_events.read(), selected_hour(), day_start,
                    settings.settings.read().segment_height,
                    settings.settings.read().selected_segment_height);

                let end_ts =
                    y_to_timestamp(end, &day_events.read(), selected_hour(), day_start,
                    settings.settings.read().segment_height,
                    settings.settings.read().selected_segment_height);

                let formated_start = convert_ts_to_local_date(start_ts).format("%H:%M:%S").to_string(); 
                let formated_end = convert_ts_to_local_date(end_ts).format("%H:%M:%S").to_string();

                let top = start.min(end);
                let height = (end - start).abs();

                rsx! {
                   div {
                        class: "absolute w-full left-0 bg-primary/20 z-50",
                        style: format!(
                            "top:{}px;height:{}px;",
                            top,
                            height
                        ),
                        if selected_start_position().is_some() && selected_end_position().is_some() && selected_start_position() != selected_end_position() {
                            div {
                                class: "absolute right-1 top-1 flex gap-0.5 text-sm",
                                if drag_state().is_none() {
                                    button {
                                        class: "p-0.5! glass rounded-full",
                                        onpointerdown: move |evt| {
                                            evt.stop_propagation();
                                        },
                                        onclick: move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            evt.prevent_default();
                                            visible_stats.set(true);
                                        },
                                        Icon {
                                            icon: LdBarChart,
                                            width: 12,
                                            height: 12
                                        }
                                    }
                                    button {
                                        class: "p-0.5! glass rounded-full",
                                        onpointerdown: move |evt| {
                                            evt.stop_propagation();
                                        },
                                        onclick: move |evt: Event<MouseData>| {
                                            evt.stop_propagation();
                                            evt.prevent_default();
                                            selected_start_position.set(None);
                                            selected_end_position.set(None);
                                        },
                                        Icon {
                                            icon: LdX,
                                            width: 12,
                                            height: 12
                                        }
                                    }
                                } else {
                                    div {
                                        class: "flex gap-1 text-xs opacity-60 pointer-events-none select-none",
                                        div {
                                            "{formated_start}"
                                        }
                                        div {
                                         "-"   
                                        }
                                        div {
                                            "{formated_end}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            {
                if settings.settings.read().show_current_time_line {
                    let y = timestamp_to_y(current_time() as u64, &day_events.read(), selected_hour(), props
                        .day
                        .read()
                        .date_naive()
                        .and_hms_opt(0, 0, 0)
                        .unwrap()
                        .and_local_timezone(Local)
                        .unwrap()
                        .timestamp_millis() as u64,
                        settings.settings.read().segment_height,
                        settings.settings.read().selected_segment_height);

                    let time = convert_ts_to_local_date(current_time() as u64).format("%H:%M:%S").to_string();
                    
                    rsx! {
                        div {
                            class: "absolute left-0 z-60 w-full flex items-center",
                            style: format!("top: {}px; transform: translateY(-50%);", y),

                            div {
                                class: "text-xs bg-primary rounded-r-md px-1 py-0.5 text-foreground shrink-0",
                                "{time}"
                            }

                            div {
                                class: "h-[2px] flex-1 bg-primary/80 rounded-md"
                            }
                        }
                    }
                } else {
                    rsx! {
                        ""
                    }
                }
            }
            {
                day_events
                    .iter()
                    .map(|segment| {
                        let start_hour = segment.start;
                        let end_hour = segment.end;
                        let hour_count = (end_hour - start_hour + 1).max(1);
                        let is_selected = selected_hour()
                            .map(|h| h >= start_hour && h <= end_hour)
                            .unwrap_or(false) && hour_count == 1;
                        let height = if is_selected { settings.settings.read().selected_segment_height as f32 } else { settings.settings.read().segment_height as f32 };
                        let start_ts = props
                            .day
                            .read()
                            .date_naive()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap()
                            .timestamp_millis();
                        let jobs = props.jobs.read();
                        let mut event_jobs: Vec<JobModel> = Vec::new();

                        let segment_st_timestamp =
                            start_ts + start_hour as i64 * 60 * 60 * 1000;

                        let segment_ed_timestamp =
                            start_ts + (end_hour as i64 + 1) * 60 * 60 * 1000;

                        jobs.iter().for_each(|job| {
                            let (st_timestamp, ed_timestamp) = if job.start_ts > 86_400_000 {
                                (
                                    job.start_ts,
                                    job.end_ts,
                                )
                            } else {
                                (
                                    start_ts + job.start_ts,
                                    start_ts + job.end_ts,
                                )
                            };

                            let c = job.cron.clone().unwrap_or("* * * * * * *".to_string());
                            let cron = CronExpr::parse(c.as_str()).unwrap_or_else(|_| CronExpr::parse("* * * * * * *").unwrap());

                            let dt = Local::now();

                            let is_today = cron.matches(dt, Some(String::from("- - - - + - -")));
                            
                            if (st_timestamp <= segment_ed_timestamp
                                && ed_timestamp >= segment_st_timestamp) && is_today
                            {
                                event_jobs.push(job.clone());
                            }

                        });

                        let is_current_hour = {
                            let now = Local::now().hour();
                            now >= start_hour && now <= end_hour
                        };

                        rsx! {
                            div {
                                key: "{start_hour}-{end_hour}", 
                                class: format!("relative border-dashed border-border/10 border-b-[1px] last:border-b-0"),

                                style: format!("height: {}px;", height as i32),

                                EventElement {
                                    key: "{start_hour}-{end_hour}",
                                    class: "h-full border-none! z-1",
                                    events: segment.group.clone(),
                                    jobs: event_jobs.clone(),
                                    selected_job,
                                    start_hour,
                                    end_hour,
                                    orientation: props.orientation.clone(),
                                    style: format!("height: {}px;", height as i32),
                                    is_selected: is_selected.clone(),
                                }

                                if hour_count > 1 {
                                    div { class: "absolute z-1 left-2 top-1 text-xs opacity-60 pointer-events-none select-none", "{start_hour}:00-{end_hour}:00"
                                 }
                                } else {
                                    div { 
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let current_hour = selected_hour();
                                            if current_hour.is_some() {
                                                if current_hour.unwrap() == start_hour {
                                                    selected_hour.set(None);
                                                } else {
                                                    selected_hour.set(Some(start_hour));
                                                }
                                            } else {
                                                selected_hour.set(Some(start_hour));
                                            }
                                        },
                                        class: format!("absolute z-1 left-{} top-1 text-xs opacity-60 select-none flex gap-1 cursor-pointer",                                         if is_selected { "2" } else { "1" }), 
                                        "{start_hour}:00 ",
                                        if is_selected {
                                            button {
                                                Icon { icon: LdArrowUpToLine, height: 10, width: 10 }
                                            }
                                        } else {
                                            button {
                                                Icon { icon: LdArrowDownToLine, height: 10, width: 10 }
                                            }
                                        }
                                    }
                                }

                                div {
                                    class: "flex justify-evenly absolute top-0 left-0 h-full w-full z-0",
                                    {(0..5).map(|i| {
                                        rsx! {
                                            div { class: "h-full w-[1px] border-dashed border-border/10 border-l-[1px] " }
                                        }
                                    })}
                                }
                                if is_selected {
                                    {(1..4).map(|i| {
                                        rsx! {
                                            div { class: "absolute left-2 z-40 text-xs opacity-60 pointer-events-none select-none z-1 -translate-y-1/2", style: format!("top: calc(100%/4*{})", i), {format!("{}:{}",start_hour, 60 / 4 * i)} }
                                        }
                                    })}
                                }

                                if is_selected {
                                    {(1..12).map(|i| {
                                        let minute = i * 5;

                                        let width = if minute % 15 == 0 {
                                            4
                                        } else if minute % 10 == 0 {
                                            3
                                        } else {
                                            2
                                        };

                                        rsx! {
                                            div {
                                                class: "absolute left-1 z-40 h-[1px] bg-border",
                                                style: format!(
                                                    "top: calc(100% / 12 * {}); width: {}px; left: {}px",
                                                    i,
                                                    width,
                                                    if is_current_hour { 4 } else { 0 }
                                                ),
                                            }
                                        }
                                    })}
                                } else {
                                    {(1..4).map(|i| {
                                        rsx! {
                                            div {
                                                class: "absolute left-0 z-40 h-[1px] bg-border",
                                                style: format!(
                                                    "top: calc(100% / 4 * {}); width: 2px; left: {}px",
                                                    i,
                                                    if is_current_hour { 4 } else { 0 }
                                                ),
                                            }
                                        }
                                    })}
                                }

                            }
                        }
                    })
            }
        }
    }
}
