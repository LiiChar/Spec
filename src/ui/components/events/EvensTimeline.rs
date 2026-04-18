use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike, Utc};
use dioxus::{html::label::form, prelude::*};

use crate::{core::EventModel, lib::{convert_ts_to_local_date, get_process_color}, ui::{EventElement, EventsList}};

#[derive(PartialEq, Clone)]
pub enum TimelineOrientation {
    Horizontal,
    Vertical,
}

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    events: Vec<EventModel>,
    orientation: TimelineOrientation
}

fn group_by_hours(
    events: &[EventModel],
) -> HashMap<u32, HashMap<u32, Vec<EventModel>>> {
    let mut result: HashMap<u32, HashMap<u32, Vec<EventModel>>> = HashMap::new();

    for e in events {
        let start = convert_ts_to_local_date(e.timestamp);
        let end = convert_ts_to_local_date(e.timestamp + e.duration);

        let mut insert = |dt: DateTime<Local>| {
            let day = dt.day();
            let hour = dt.hour();

            result
                .entry(day)
                .or_default()
                .entry(hour)
                .or_default()
                .push(e.clone());
        };

        insert(start);

        // если пересекает границы
        if start.hour() != end.hour() || start.day() != end.day() {
            insert(end);
        }
    }

    result
}

/// Получить уникальные процессы для легенды
fn get_unique_processes(events: &[EventModel]) -> Vec<String> {
    let mut processes: Vec<String> = events
        .iter()
        .map(|e| e.window.process_name.clone())
        .collect();
    processes.sort();
    processes.dedup();
    processes
}

#[component]
pub fn EventsTimeline(props: EventsCalendarProps) -> Element {
    let week = group_by_hours(&props.events);
    let days = Local::now().date_naive().day();
    let day_data = week.get(&days);

    let mut selected_hour: Signal<Option<u32>> = use_signal(|| None);

    rsx! {
        div {
            class: format!("flex flex-col gap-0.5 w-full h-full items-stretch relative transition-all duration-300 {}", 
                match props.orientation {
                    TimelineOrientation::Horizontal => "flex-row",
                    TimelineOrientation::Vertical => "flex-col"
                }
            ),
            
            {(0..24).map(|hour| {
                let empty: Vec<EventModel> = Vec::new();

                let events = day_data
                    .and_then(|h| h.get(&hour))
                    .unwrap_or(&empty);
                    
                rsx! {
                    EventElement { 
                    class: "min-h-[100px] current-hour",
                        events: events.clone(), 
                        hour, 
                        orientation: props.orientation.clone(), 
                        onclick: Some(EventHandler::new(move |_| {
                            selected_hour.set(Some(hour));
                        })) 
                    }
                }
            })}
        }
    }
}