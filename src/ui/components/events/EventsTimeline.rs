use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use dioxus::prelude::*;

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

    rsx! {
        div {
            class: format!(
                "flex w-full h-full rounded-sm overflow-hidden border-border border-[1px] {}",
                match props.orientation {
                    TimelineOrientation::Horizontal => "flex-row",
                    TimelineOrientation::Vertical => "flex-col",
                }
            ),

            {(0..24).map(|hour| {
                let hour_events = day_events.get(&hour).cloned().unwrap_or_default();

                rsx! {
                    div { 
                        onclick: move |_| selected_hour.set(Some(hour)),
                        EventElement {
                            key: "{hour}",
                            class: format!("min-h-[80px] rounded-none! border-t-0! border-l-0! border-r-0! {}", if selected_hour() == Some(hour) { "min-h-[800px]" } else { "" }),
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
