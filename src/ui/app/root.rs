use chrono::Local;
use dioxus::prelude::*;

use crate::{
    RX,
    core::with_database,
    lib::convert_ts_to_local_date,
    ui::{
        provide_alert, provide_app, provide_settings, provide_toast, use_app, INITIAL_EVENT_LIMIT,
        Layout, MAX_EVENTS_IN_MEMORY, Router, Tray,
    },
};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn Root() -> Element {
    provide_settings();
    provide_app();
    provide_toast();
    provide_alert();

    let context = use_app();

    let mut events = context.events;
    let mut jobs = context.jobs;
    let day = context.day;
    let time = context.time;
    let start_time = context.start_time;

    let mut did_start = use_signal(|| false);

    // ---------- load events ----------
    {
        let selected_day = day.read().date_naive();
        let selected_time = *time.read();
        let selected_start_time = *start_time.read();

        use_effect(move || {
            spawn(async move {
                if let Some(start) = selected_start_time {
                    let end = selected_time;

                    let result = tokio::task::spawn_blocking(move || {
                        with_database(|db| db.get_events_in_range(start, end))
                    })
                    .await
                    .unwrap();

                    if let Ok(events_loaded) = result {
                        events.set(events_loaded);
                    }

                    return;
                }

                let start_of_day = selected_day
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(Local)
                    .unwrap()
                    .timestamp_millis();

                let result = tokio::task::spawn_blocking(move || {
                    with_database(|db| db.get_events_since(start_of_day, INITIAL_EVENT_LIMIT))
                })
                .await
                .unwrap();

                if let Ok(events_loaded) = result {
                    events.set(events_loaded);
                }
            });
        });
    }

    // ---------- load jobs ----------
    {
        let selected_day = day.read().date_naive();

        use_effect(move || {
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
                    with_database(|db| db.get_jobs_for_day(start_of_day, end_of_day))
                })
                .await
                .unwrap();

                if let Ok(jobs_loaded) = result {
                    jobs.set(jobs_loaded);
                }
            });
        });
    }

    // ---------- live events ----------
    use_effect(move || {
        if did_start() {
            return;
        }

        did_start.set(true);

        let rx_opt = RX
            .lock()
            .ok()
            .and_then(|rx_guard| rx_guard.as_ref().cloned());

        if let Some(rx) = rx_opt {
            spawn(async move {
                loop {
                    let recv_result = tokio::task::spawn_blocking({
                        let rx = rx.clone();
                        move || rx.recv()
                    })
                    .await
                    .unwrap();

                    match recv_result {
                        Ok(event) => {
                            let selected_day = day.read().date_naive();
                            let event_day =
                                convert_ts_to_local_date(event.timestamp).date_naive();

                            if event_day != selected_day {
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
            Router {}
        }
    }
}