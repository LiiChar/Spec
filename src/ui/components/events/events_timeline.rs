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
    core::{database::database, Database, EventModel, JobModel},
    lib::{convert_ts_to_local_date, merge_events},
    ui::{EventElement, JobFormModal, JobModal},
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
    let day_events = use_memo(move || {
        let merged = {
            let items = props.events.read();
            merge_events(&items)
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

    let mut selected_hour = use_signal(|| None::<u32>);
    let mut selected_job = use_signal(|| None::<JobModel>);

    // let mut on_mouse_down = use_signal(|| false);
    // let mut start_mouse_position = use_signal(|| (0.0, 0.0));
    // let mut end_mouse_position = use_signal(|| (0.0, 0.0));
    // let mut timeline_height = use_signal(|| 0.0);
    // let mut container_rect = use_signal(|| Rect::zero() as Rect<f64, Pixels>);
    // let mut show_job_form = use_signal(|| false);
    // let mut selected_job_range = use_signal(|| (0i64, 0i64));

    rsx! {
        JobModal {
            job: selected_job.read().clone(),
            on_close: move |_| selected_job.set(None),
        }
        // JobFormModal {
        //     visible: show_job_form,
        //     start_ts: selected_job_range().0,
        //     end_ts: selected_job_range().1,
        //     on_save: move |job: JobModel| {
        //         let _ = Database::new(DATABASE_PATH)
        //             .insert_jobs(&job);
        //         show_job_form.set(false);
        //     },
        //     on_cancel: move |_| {
        //         show_job_form.set(false);
        //     }
        // }
        div {
            // onmounted: move |e| {
            //     async move {
            //         let rect = e.get_client_rect().await.unwrap_or_default();
            //         timeline_height.set(rect.size.height);
            //         container_rect.set(rect);
            //     }
            // },
            // onmousedown: move |evt| {
            //     on_mouse_down.set(true);

            //     let x: f64 = evt.page_coordinates().x as f64 - container_rect.read().origin.x as f64;
            //     let y: f64 = evt.page_coordinates().y as f64 - container_rect.read().origin.y as f64;
            //     start_mouse_position.set((x, y + 18.0));
            //     end_mouse_position.set((x, y + 18.0));
            // },
            // onmousemove: move |evt| {
            //     if on_mouse_down() {
            //         evt.stop_propagation();
            //         evt.prevent_default();
            //         let x: f64 = evt.page_coordinates().x as f64 - container_rect.read().origin.x as f64;
            //         let y: f64 = evt.page_coordinates().y as f64 - container_rect.read().origin.y as f64;
            //         end_mouse_position.set((x, y + 18.0));
            //     }
            // },
            // onmouseup: move |evt| {
            //     let x: f64 = evt.page_coordinates().x as f64 - container_rect.read().origin.x as f64;
            //     let y: f64 = evt.page_coordinates().y as f64 - container_rect.read().origin.y as f64;
            //     end_mouse_position.set((x, y + 18.0));
            //     on_mouse_down.set(false);
            // },
            class: format!(
                "flex w-full h-min relative rounded-md overflow-hidden border border-border/50 bg-card/30 user-select-none {}",
                match props.orientation {
                    TimelineOrientation::Horizontal => "flex-row",
                    TimelineOrientation::Vertical => "flex-col",
                },
            ),

            // Keyboard navigation container
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

            // if on_mouse_down() || end_mouse_position() != start_mouse_position() {
            //     div {
            //         onmousedown: move |evt| evt.stop_propagation(),
            //         onmouseup: move |evt| evt.stop_propagation(),
            //         class: "absolute bg-primary/20 z-30",
            //         style: match props.orientation {
            //             TimelineOrientation::Horizontal => {
            //                 let start_x: i32 = start_mouse_position().0.floor() as i32;
            //                 let end_x: i32 = end_mouse_position().0.floor() as i32;
            //                 let left = start_x.min(end_x);
            //                 let width = (end_x - start_x).abs();
            //                 format!("width: {}px; height: 100%; left: {}px; top: 0;", width, left)
            //             }
            //             TimelineOrientation::Vertical => {
            //                 let start_y: i32 = start_mouse_position().1.floor() as i32;
            //                 let end_y: i32 = end_mouse_position().1.floor() as i32;
            //                 let top = start_y.min(end_y);
            //                 let height = (end_y - start_y).abs();
            //                 format!("height: {}px; width: 100%; top: {}px; left: 0;", height, top)
            //             }
            //         },
            //         {
            //             // Calculate virtual total height based on 24 hours
            //             let base_hour_height = 80.0;
            //             let expanded_hour_height = 800.0;

            //             let total_height: f64 = (0..24)
            //                 .map(|h| {
            //                     if selected_hour() == Some(h) {
            //                         expanded_hour_height
            //                     } else {
            //                         base_hour_height
            //                     }
            //                 })
            //                 .sum();

            //             // Get Y positions based on orientation
            //             let (start_pos, end_pos): (f64, f64) = match props.orientation {
            //                 TimelineOrientation::Horizontal => {
            //                     (start_mouse_position().0, end_mouse_position().0)
            //                 }
            //                 TimelineOrientation::Vertical => {
            //                     (start_mouse_position().1, end_mouse_position().1)
            //                 }
            //             };

            //             // Clamp to virtual timeline bounds
            //             let start_y = start_pos.clamp(0.0, total_height);
            //             let end_y = end_pos.clamp(0.0, total_height);

            //             // Day start timestamp
            //             let day_start = props.day.read()
            //                 .date_naive()
            //                 .and_hms_opt(0, 0, 0)
            //                 .unwrap()
            //                 .and_local_timezone(Local)
            //                 .unwrap()
            //                 .timestamp_millis() as u64;

            //             // Convert Y positions to timestamps using virtual coordinate system
            //             let mut from = y_to_timestamp(start_y, selected_hour(), day_start);
            //             let mut to = y_to_timestamp(end_y, selected_hour(), day_start);

            //             // Swap if dragged in reverse direction
            //             if from > to {
            //                 std::mem::swap(&mut from, &mut to);
            //             }

            //             let format_start_date = convert_ts_to_local_date(from).format("%d.%m.%Y %H:%M:%S").to_string();
            //             let format_end_date = convert_ts_to_local_date(to).format("%d.%m.%Y %H:%M:%S").to_string();

            //             println!("Selected range: {:?} {:?} (total_height: {:?})", format_start_date, format_end_date, total_height);

            //             let selected_events = props.events
            //                 .read()
            //                 .iter()
            //                 .cloned()
            //                 .filter(|e| {
            //                     let event_start = e.timestamp;
            //                     let event_end = e.timestamp + e.duration;

            //                     event_start < to && event_end > from
            //                 })
            //                 .collect::<Vec<_>>();

            //             rsx! {

            //                 div {
            //                     class: "flex flex-col gap-1 w-1/3 text-xs h-full overflow-y-auto",
            //                     {
            //                         selected_events.clone().iter().map(|e| {
            //                             if let Some(w) = &e.window {
            //                                 rsx! {
            //                                     div {
            //                                         class: "flex items-center justify-center",
            //                                         div {
            //                                             "{w.title}"
            //                                         }
            //                                     }
            //                                 }
            //                             } else {
            //                                 rsx! {
            //                                     ""
            //                                 }
            //                             }
            //                         })
            //                     }
            //                 }
            //                 div {
            //                     class: "absolute top-1 right-1 bg-background",
            //                     button {
            //                         class: "p-1 hover:bg-muted rounded",
            //                         onclick: move |_| {
            //                             selected_job_range.set((from as i64, to as i64));
            //                             show_job_form.set(true);
            //                         },
            //                         Icon {icon: LdPlus}
            //                     }
            //                 }
            //             }
            //         }
            //     }
            // }

            // {
            //     props.jobs.read().clone().iter().map(|job| {
            //         let day_start: i64 = props.day.read()
            //             .date_naive()
            //             .and_hms_opt(0, 0, 0)
            //             .unwrap()
            //             .and_local_timezone(Local)
            //             .unwrap()
            //             .timestamp_millis();
            //         let job_start = job.start_ts - day_start.clone();
            //         let job_end = job.end_ts - day_start.clone();

            //         // Calculate position based on 24 hours with base height 80px per hour
            //         let total_height: f64 = 80.0 * 24.0;
            //         let hour_height = 80.0;

            //         let start_y = (job_start / 3_600_000) as f64 * hour_height;
            //         let end_y = (job_end / 3_600_000) as f64 * hour_height;
            //         let height = (end_y - start_y).max(20.0);

            //         rsx! {
            //             div {
            //                 onmousedown: move |evt| evt.stop_propagation(),
            //                 onmouseup: move |evt| evt.stop_propagation(),
            //                 class: format!("absolute left-2 right-2 z-10 rounded px-2 py-1 text-white text-xs z-5 flex justify-end"),
            //                 style: format!("top: {}px; height: {}px; background-color: {}", start_y as i32, height as i32, job.color),
            //                 div {
            //                     class: "relative z-40",
            //                     div {
            //                         class: "font-medium truncate",
            //                         "{job.name}"
            //                     }
            //                     if let Some(desc) = &job.description {
            //                         div {
            //                             class: "text-xs opacity-80 truncate",
            //                             "{desc}"
            //                         }
            //                     }
            //                 }
            //             }
            //         }
            //     })
            // }
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
                        jobs
                        .iter()
                            .for_each(|job| {
                                let st_timestamp = start_ts + job.start_ts * 1000 as i64;
                                let ed_timestamp = start_ts + job.end_ts * 1000 as i64;
                                let segment_st_timestamp = start_ts
                                    + start_hour as i64 * 60 * 60 * 1000;
                                let segment_ed_timestamp = start_ts
                                    + end_hour as i64 * 60 * 60 * 1000;
                                if st_timestamp <= segment_ed_timestamp
                                    && ed_timestamp >= segment_st_timestamp
                                {
                                    event_jobs.push(job.clone());
                                }
                            });
                        rsx! {
                            div {
                                class: "relative",
                                onclick: move |_| selected_hour.set(Some(start_hour)),

                                style: format!("height: {}px;", height as i32),

                                EventElement {
                                    key: "{start_hour}-{end_hour}",
                                    class: "h-full border-none!",
                                    events: segment.group.clone(),
                                    jobs: event_jobs.clone(),
                                    selected_job,
                                    start_hour,
                                    end_hour,
                                    orientation: props.orientation.clone(),
                                    style: format!("height: {}px;", height as i32),
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
                                    div { class: "absolute left-1 top-1 text-xs opacity-60", "{start_hour}-{end_hour}" }
                                } else {
                                    div { class: "absolute left-1 top-1 text-xs opacity-60", "{start_hour}" }
                                }
                            }
                        }
                    })
            }
        }

    }
}
