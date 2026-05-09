use std::collections::HashMap;

use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Timelike};
use dioxus::{
    html::geometry::{euclid::Rect, Pixels},
    prelude::*,
};
use dioxus_free_icons::icons::ld_icons::{LdArrowUpToLine, LdPlus};
use dioxus_free_icons::Icon;

use crate::{
    config::DATABASE_PATH,
    core::{Database, EventModel, EventType, JobModel, database::database},
    lib::{convert_ts_to_local_date, merge_events, merge_visual_density},
    ui::{EventElement, JobFormModal, JobModal, use_settings},
};

#[derive(PartialEq, Clone)]
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

fn y_to_timestamp(y: f64, selected_hour: Option<u32>, day_start: u64) -> u64 {
    let base = 80.0;
    let expanded = 800.0;

    let mut acc = 0.0;

    for hour in 0..24 {
        let h_size = if selected_hour == Some(hour) {
            expanded
        } else {
            base
        };

        if y <= acc + h_size {
            let inside = (y - acc) / h_size;

            let ms_in_hour = (inside * 3_600_000.0) as u64;

            return day_start + hour as u64 * 3_600_000 + ms_in_hour;
        }

        acc += h_size;
    }

    day_start + 86_400_000
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

            merge_visual_density(merged, px_per_hour, 3.0)
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



    rsx! {
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
            div {
                tabindex: 0,
                onkeydown: move |evt| {
                    match evt.key() {
                        Key::Home => {
                            evt.prevent_default();
                            selected_hour.set(Some(0));
                        }
                        Key::End => {
                            evt.prevent_default();
                            selected_hour.set(Some(23));
                        }
                        Key::Escape => {
                            evt.prevent_default();
                            selected_hour.set(None);
                        }
                        _ => {}
                    }
                },
                onclick: move |_| {},
                class: "absolute inset-0",
                role: "application",
                aria_label: "Таймлайн активности. Используйте стрелки вверх/вниз или k/j для навигации по часам",
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
                        let height = if is_selected { 800.0 } else { 80.0 };
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

                            if st_timestamp < segment_ed_timestamp
                                && ed_timestamp >= segment_st_timestamp
                            {
                                event_jobs.push(job.clone());
                            }
                        });

                        rsx! {
                            div {
                                class: "relative border-dashed border-border/10 border-b-[1px] last:border-b-0",
                                onclick: move |_| selected_hour.set(Some(start_hour)),

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

                                if is_selected {
                                    button {
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            selected_hour.set(None);
                                        },
                                        class: "absolute z-40  border border-border/0 hover:border-border/40 rounded-md p-1 right-1 top-1 hover:bg-background/50 hover:backdrop-blur-lg text-xs opacity-60",
                                        Icon { icon: LdArrowUpToLine, height: 12, width: 12 }
                                    }
                                }



                                // подпись диапазона (если сегмент > 1 часа)
                                if hour_count > 1 {
                                    div { class: "absolute z-2 left-2 top-1 text-xs opacity-60", "{start_hour}:00-{end_hour}:00" }
                                } else {
                                    div { class: "absolute z-2 left-1 top-1 text-xs opacity-60", "{start_hour}:00" }
                                }

                                div {
                                    class: "flex justify-evenly absolute top-0 left-0 h-full w-full z-0",
                                    {(0..5).map(|i| {
                                        rsx! {
                                            div { class: "h-full w-[1px] border-dashed border-border/10 border-l-[1px]" }
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
