use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use dioxus::prelude::*;

use crate::{
    RX, config::DATABASE_PATH, core::{EventModel, WindowsDatabase, tray}, lib::convert_ts_to_local_date, ui::{Events, Layout, Tray}
};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const INITIAL_EVENT_LIMIT: i64 = 2_500;
const MAX_EVENTS_IN_MEMORY: usize = 5_000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    pub theme: Theme,
    pub day: Signal<DateTime<Local>>,
    pub events: Signal<Vec<EventModel>>
} 

impl Default for AppContext {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            events: Signal::new(Vec::new()),
            day: Signal::new(Local::now()),
        }
    }
}

#[component]
pub fn Root() -> Element {
    use_context_provider(|| AppContext::default());

    let context = use_context::<AppContext>();
    let mut events = context.events;
    let day = context.day;
    let mut did_start = use_signal(|| false);

    use_effect(move || {
        let selected_day = day.read().date_naive();

        spawn(async move {
            let start_of_day = selected_day
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp_millis();

            let result = tokio::task::spawn_blocking(move || {
                WindowsDatabase::new(DATABASE_PATH)
                    .get_events_since(start_of_day, INITIAL_EVENT_LIMIT)
            })
            .await
            .unwrap();

            if let Ok(events_loaded) = result {
                events.set(events_loaded);
            }
        });
    });

    use_effect(move || {
        if did_start() {
            return;
        }
        did_start.set(true);

        let rx_opt = RX.lock().ok().and_then(|rx_guard| rx_guard.as_ref().cloned());
        if let Some(rx) = rx_opt {
            spawn(async move {
                let start_of_day_ts = day.read()
                    .with_hour(0)
                    .and_then(|dt| dt.with_minute(0))
                    .and_then(|dt| dt.with_second(0))
                    .and_then(|dt| dt.with_nanosecond(0))
                    .map(|dt| dt.timestamp_millis())
                    .unwrap_or_default();

                let load_result = tokio::task::spawn_blocking(move || {
                    WindowsDatabase::new(DATABASE_PATH)
                        .get_events_since(start_of_day_ts, INITIAL_EVENT_LIMIT)
                })
                .await
                .unwrap();

                if let Ok(loaded_events) = load_result {
                    events.set(loaded_events);
                }

                loop {
                    let recv_result = tokio::task::spawn_blocking({
                        let rx = rx.clone();
                        move || rx.recv()
                    })
                    .await
                    .unwrap();

                   match recv_result {
                        Ok(event) => {
                            let selected_date = day.read().date_naive();
                            let event_date = convert_ts_to_local_date(event.timestamp).date_naive();
                            if event_date != selected_date {
                                continue;
                            }
                            events.with_mut(|ev| {
                                ev.push(event);

                                if ev.len() > MAX_EVENTS_IN_MEMORY {
                                    let overflow = ev.len() - MAX_EVENTS_IN_MEMORY;
                                    ev.drain(0..overflow);
                                }
                            });
                        }
                        Err(_) => break,
                    }
                }
            });
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        Tray {}

        Layout {
            div {
                class: "flex gap-0 h-full w-full relative",
                div {
                    class: "flex-1 flex flex-col",
                    Events { }
                }
            }
        }
    }
}
