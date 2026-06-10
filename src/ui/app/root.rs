use chrono::Local;
use dioxus::prelude::*;

use crate::{
    core::{tag_repo::auto_tag, with_database},
    lib::convert_ts_to_local_date,
    ui::{
        INITIAL_EVENT_LIMIT, Layout, MAX_EVENTS_IN_MEMORY, Router, Tray, db, provide_alert, provide_app, provide_db, provide_event_bus, provide_settings, provide_toast, use_app, use_event_bus
    },
};

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
pub fn Root() -> Element {
    provide_db();
    provide_settings();
    provide_app();
    provide_toast();
    provide_alert();
    provide_event_bus();

    let context = use_app();

    let mut events = context.events;
    let mut jobs = context.jobs;
    let mut tags = context.tags;
    
    let day = context.day;
    let time = context.time;
    let start_time = context.start_time;

    let mut did_start = use_signal(|| false);

    // ---------- load events ----------
    {
        use_effect(move || {
            // read signals inside the effect so it re-runs when they change
            let selected_day = day.read().date_naive();
            let selected_time = *time.read();
            let selected_start_time = *start_time.read();

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
                    with_database(|db| db.get_jobs_for_day(start_of_day, end_of_day))
                })
                .await
                .unwrap();

                if let Ok(jobs_loaded) = result {
                    jobs.set(jobs_loaded);
                }

                let result = tokio::task::spawn_blocking(move || {
                    with_database(|db| db.get_tags())
                })
                .await
                .unwrap();

                if let Ok(tags_loaded) = result {
                    tags.set(tags_loaded);
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

        let event_bus = use_event_bus();
        let rx = event_bus.0;

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
                            let event = with_database(|db| {
                                db.auto_tag(event)
                            }).expect("Failed auto tag event");

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