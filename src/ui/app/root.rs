use chrono::{DateTime, Datelike, Local, NaiveDate, Timelike};
use dioxus::prelude::*;

use crate::{
    RX, config::DATABASE_PATH, core::{EventModel, JobModel, WindowsDatabase, tray}, lib::convert_ts_to_local_date, ui::{Events, Layout, Tray}
};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const INITIAL_EVENT_LIMIT: i64 = 10_500;
const MAX_EVENTS_IN_MEMORY: usize = 15_000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppContext {
    pub theme: Theme,
    pub day: Signal<DateTime<Local>>,
    pub time: Signal<i64>,
    pub start_time: Signal<Option<i64>>,
    pub events: Signal<Vec<EventModel>>,
    pub jobs: Signal<Vec<JobModel>>,
} 

impl Default for AppContext {
    
    fn default() -> Self {
        let now = Local::now().timestamp_millis();
            
        Self {
            theme: Theme::Light,
            events: Signal::new(Vec::new()),
            day: Signal::new(Local::now()),
            time: Signal::new(now as i64),
            start_time: Signal::new(None),
            jobs: Signal::new(Vec::new()),
        }
    }
}

#[component]
pub fn Root() -> Element {
    use_context_provider(|| AppContext::default());

    let context = use_context::<AppContext>();
    let mut events = context.events;
    let mut jobs = context.jobs;
    let day = context.day;
    let time = context.time;
    let start_time = context.start_time;
    let mut did_start = use_signal(|| false);

    use_effect(move || {
        let selected_time = *time.read(); // копируем i64
        let selected_start_time = *start_time.read(); // копируем Option<i64>

        if selected_start_time.is_none() {
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
                    println!("Events loaded: {:?}", events_loaded.len());
                    events.set(events_loaded);
                }
            });

            return;
        }

        let start = selected_start_time.unwrap();
        let end = selected_time;

        spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                WindowsDatabase::new(DATABASE_PATH)
                    .get_events_in_range(start, end)
            })
            .await
            .unwrap();

            if let Ok(events_loaded) = result {
                println!("Events loaded: {:?}", events_loaded.len());
                events.set(events_loaded);
            }
        });
    });

    // Load jobs for the selected day
    use_effect(move || {
        let selected_day = day.read().date_naive();

        spawn(async move {
            let start_of_day = selected_day
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp_millis();
            
            let end_of_day = selected_day
                .and_hms_opt(23, 59, 59)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap()
                .timestamp_millis();

            let result = tokio::task::spawn_blocking(move || {
                WindowsDatabase::new(DATABASE_PATH)
                    .get_jobs_for_day(start_of_day, end_of_day)
            })
            .await
            .unwrap();

            if let Ok(jobs_loaded) = result {
                println!("Jobs loaded: {:?}", jobs_loaded.len());
                jobs.set(jobs_loaded);
            }
        });
    });

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
                println!("Events loaded: {:?}", events_loaded.len());
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
