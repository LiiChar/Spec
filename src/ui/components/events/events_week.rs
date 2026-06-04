use std::collections::HashMap;

use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike, Utc};
use dioxus::{html::label::form, prelude::*};

use crate::{core::EventModel, lib::convert_ts_to_local_date};

#[derive(Props, PartialEq, Clone)]
pub struct EventsCalendarProps {
    events: Vec<EventModel>,
}

fn is_today(date: NaiveDate) -> bool {
    date == Local::now().date_naive()
}

fn group_by_hours(events: &[EventModel]) -> HashMap<u32, HashMap<u32, Vec<EventModel>>> {
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

pub fn build_week(date: NaiveDate) -> Vec<NaiveDate> {
    let mut current = date - chrono::Duration::days(6);

    let mut week = Vec::new();

    for _ in 0..7 {
        week.push(current);
        current = current.succ_opt().unwrap();
    }

    week
}

#[component]
pub fn EventsWeek(props: EventsCalendarProps) -> Element {
    let week = group_by_hours(&props.events);
    let days = build_week(chrono::Local::now().date_naive());

    rsx! {
        div { class: "flex flex-row gap-1 w-full h-full p-1",

            {
                days.iter()
                    .map(|day| {
                        let day_key = day.day();
                        let day_data = week.get(&day_key);
                        rsx! {
                            div { key: "{day_key}", class: "flex flex-col gap-0.5 w-full items-center",

                                {
                                    (1..24)
                                        .map(|hour| {
                                            let empty: Vec<EventModel> = Vec::new();
                                            let events = day_data.and_then(|h| h.get(&hour)).unwrap_or(&empty);
                                            rsx! {
                                                div {
                                                    key: "{hour}",
                                                    class: format!(
                                                        "w-full h-[30px] rounded-sm relative {}",
                                                        if !events.is_empty() { "bg-blue-500/30" } else { "bg-zinc-800/30" },

                                                        // 🔥 правильная математика
                                                    ),

                                                    span { class: "absolute left-1 text-[10px] opacity-50", "{hour}" }

                                                    {
                                                        events
                                                            .iter()
                                                            .map(|e| {
                                                                let start_dt = convert_ts_to_local_date(e.timestamp);
                                                                let end_dt = convert_ts_to_local_date(e.timestamp + e.duration);
                                                                let start_sec = start_dt.minute() * 60 + start_dt.second();
                                                                let end_sec = end_dt.minute() * 60 + end_dt.second();
                                                                let top = (start_sec as f32 / 3600.0) * 100.0;
                                                                let height = ((end_sec.saturating_sub(start_sec)) as f32 / 3600.0)
                                                                    * 100.0;
                                                                let window_title = e
                                                                    .window
                                                                    .as_ref()
                                                                    .map(|w| w.title.clone())
                                                                    .unwrap_or_else(|| "N/A".to_string());
                                                                rsx! {
                                                                    div {
                                                                        class: "absolute group left-0 w-full bg-secondary/70 rounded-sm",
                                                                        style: format!("top: {}%; height: {}%;", top, height.max(2.0)),
                                                                        title: format!("{} - {}", start_dt.time(), end_dt.time()),
                                                                        span { class: "group-hover:block hidden ", "{window_title}" }
                                                                    }
                                                                }
                                                            })
                                                    }
                                                }
                                            }
                                        })
                                }
                            }
                        }
                    })
            }
        }
    }
}
