use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, Timelike};
use dioxus::{html::geometry::{Pixels, euclid::Rect}, prelude::*};
use dioxus_free_icons::icons::ld_icons::{LdPlus};
use dioxus_free_icons::Icon;

use crate::{
    core::EventModel,
    lib::{convert_ts_to_local_date, merge_events},
    ui::EventElement,
};

#[derive(PartialEq, Clone)]
pub enum TimelineOrientation {
    Horizontal,
    Vertical,
}

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    events: ReadSignal<Vec<EventModel>>,
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

fn y_to_timestamp(
    y: f64,
    selected_hour: Option<u32>,
    day_start: u64,
) -> u64 {
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
pub fn EventsTimeline(props: EventsCalendarProps) -> Element {
    let day_events = use_memo(move || {
        let merged = {
            let items = props.events.read();
            merge_events(&items)
        };

        let grouped = group_by_hours(&merged);
        let today = props.day.read();

        grouped.get(&today.day()).cloned().unwrap_or_default()
    });

    let day_events = day_events();

    let mut selected_hour = use_signal(|| None::<u32>);

    let mut on_mouse_down = use_signal(|| false);
    let mut start_mouse_position = use_signal(|| (0.0, 0.0));
    let mut end_mouse_position = use_signal(|| (0.0, 0.0));
    let mut timeline_height = use_signal(|| 0.0);
    let mut container_rect = use_signal(|| Rect::zero() as Rect<f64, Pixels>);

    rsx! {
        div {
            onmounted: move |e| {
                async move {
                    let rect = e.get_client_rect().await.unwrap_or_default();
                    timeline_height.set(rect.size.height);
                    container_rect.set(rect);
                }
            },
            onmousedown: move |evt| {
                on_mouse_down.set(true);
                
                let x: f64 = evt.page_coordinates().x as f64 - container_rect.read().origin.x as f64;
                let y: f64 = evt.page_coordinates().y as f64 - container_rect.read().origin.y as f64;
                start_mouse_position.set((x, y + 18.0));
                end_mouse_position.set((x, y + 18.0));
            },
            onmousemove: move |evt| {
                if on_mouse_down() {
                    evt.stop_propagation();
                    evt.prevent_default();
                    let x: f64 = evt.page_coordinates().x as f64 - container_rect.read().origin.x as f64;
                    let y: f64 = evt.page_coordinates().y as f64 - container_rect.read().origin.y as f64;
                    end_mouse_position.set((x, y + 18.0));
                }
            },
            onmouseup: move |evt| {
                let x: f64 = evt.page_coordinates().x as f64 - container_rect.read().origin.x as f64;
                let y: f64 = evt.page_coordinates().y as f64 - container_rect.read().origin.y as f64;
                end_mouse_position.set((x, y + 18.0));
                on_mouse_down.set(false);
            },
            class: format!(
                "flex w-full h-full relative rounded-sm overflow-hidden border-border border-[1px] user-select-none {}",
                match props.orientation {
                    TimelineOrientation::Horizontal => "flex-row",
                    TimelineOrientation::Vertical => "flex-col",
                }
            ),

            if on_mouse_down() || end_mouse_position() != start_mouse_position() {
                div {
                    onmousedown: move |evt| evt.stop_propagation(),
                    onmouseup: move |evt| evt.stop_propagation(),
                    class: "absolute bg-primary/20 z-10",
                    style: match props.orientation {
                        TimelineOrientation::Horizontal => {
                            let start_x: i32 = start_mouse_position().0.floor() as i32;
                            let end_x: i32 = end_mouse_position().0.floor() as i32;
                            let left = start_x.min(end_x);
                            let width = (end_x - start_x).abs();
                            format!("width: {}px; height: 100%; left: {}px; top: 0;", width, left)
                        }
                        TimelineOrientation::Vertical => {
                            let start_y: i32 = start_mouse_position().1.floor() as i32;
                            let end_y: i32 = end_mouse_position().1.floor() as i32;
                            let top = start_y.min(end_y);
                            let height = (end_y - start_y).abs();
                            format!("height: {}px; width: 100%; top: {}px; left: 0;", height, top)
                        }
                    },
                    {
                        // Calculate virtual total height based on 24 hours
                        let base_hour_height = 80.0;
                        let expanded_hour_height = 800.0;

                        let total_height: f64 = (0..24)
                            .map(|h| {
                                if selected_hour() == Some(h) {
                                    expanded_hour_height
                                } else {
                                    base_hour_height
                                }
                            })
                            .sum();

                        // Get Y positions based on orientation
                        let (start_pos, end_pos): (f64, f64) = match props.orientation {
                            TimelineOrientation::Horizontal => {
                                (start_mouse_position().0, end_mouse_position().0)
                            }
                            TimelineOrientation::Vertical => {
                                (start_mouse_position().1, end_mouse_position().1)
                            }
                        };

                        // Clamp to virtual timeline bounds
                        let start_y = start_pos.clamp(0.0, total_height);
                        let end_y = end_pos.clamp(0.0, total_height);

                        // Day start timestamp
                        let day_start = props.day.read()
                            .date_naive()
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_local_timezone(Local)
                            .unwrap()
                            .timestamp_millis() as u64;

                        // Convert Y positions to timestamps using virtual coordinate system
                        let mut from = y_to_timestamp(start_y, selected_hour(), day_start);
                        let mut to = y_to_timestamp(end_y, selected_hour(), day_start);

                        // Swap if dragged in reverse direction
                        if from > to {
                            std::mem::swap(&mut from, &mut to);
                        }

                        let format_start_date = convert_ts_to_local_date(from).format("%d.%m.%Y %H:%M:%S").to_string();
                        let format_end_date = convert_ts_to_local_date(to).format("%d.%m.%Y %H:%M:%S").to_string();

                        println!("Selected range: {:?} {:?} (total_height: {:?})", format_start_date, format_end_date, total_height);

                        let selected_events = props.events
                            .read()
                            .iter()
                            .cloned()
                            .filter(|e| {
                                let event_start = e.timestamp;
                                let event_end = e.timestamp + e.duration;

                                event_start < to && event_end > from
                            })
                            .collect::<Vec<_>>();

                        rsx! {
                            
                            div {
                                class: "flex flex-col gap-1 w-1/3 text-xs h-full overflow-y-auto",
                                {
                                    selected_events.clone().iter().map(|e| {
                                        if let Some(w) = &e.window {
                                            rsx! {
                                                div {
                                                    class: "flex items-center justify-center",
                                                    div {
                                                        "{w.title}"
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! {
                                                ""
                                            }
                                        }
                                    })
                                }
                            }
                            div {
                                class: "absolute top-1 right-1 bg-background",
                                div {
                                    Icon {icon: LdPlus}
                                }
                            }
                        }
                    }
                }
            }
            {(0..24).map(|hour| {
                let hour_events = day_events.get(&hour).cloned().unwrap_or_default();

                rsx! {
                    div { 
                        onclick: move |_| selected_hour.set(Some(hour)),
                        EventElement {
                            key: "{hour}",
                            class: format!("min-h-[80px] relative rounded-none! border-none! {}", if selected_hour() == Some(hour) { "min-h-[800px]" } else { "" }),
                            events: hour_events,
                            hour,
                            orientation: props.orientation.clone(),
                    }
                }
            }
            })}
        }
    }
}
