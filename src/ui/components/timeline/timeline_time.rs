use chrono::Local;
use dioxus::prelude::*;

use crate::{lib::{Segment, convert_ts_to_local_date, timestamp_to_y}, ui::context::{use_app, use_settings}};
use tokio::time::Duration;

#[derive(Props, PartialEq, Clone)]
pub struct TimelineTimeProps {
    pub segments: Vec<Segment>,
    pub selected_hour: Signal<Option<u32>>,
}

#[component]
pub fn TimelineTime(props: TimelineTimeProps) -> Element {
    let app = use_app();
    let settings = use_settings();

    let mut current_time = use_signal(|| chrono::Local::now().timestamp_millis());

    use_effect(move || {
        if !settings.settings.read().show_current_time_line {
            return;
        }
        spawn(async move {
            loop {
                current_time.set(chrono::Local::now().timestamp_millis());
                tokio::time::sleep(Duration::from_millis(1000)).await;
            }
        });
    });

    let y = timestamp_to_y(
        current_time() as u64, 
        &props.segments, 
        (props.selected_hour)(), 
        app.day
            .read()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis() as u64,
        settings.settings.read().segment_height,
        settings.settings.read().selected_segment_height
    );

    let time = convert_ts_to_local_date(current_time() as u64).format("%H:%M:%S").to_string();

    if settings.settings.read().show_current_time_line {
        rsx! {
            div {
                class: "absolute left-0 z-60 w-full flex items-center",
                style: format!("top: {}px; transform: translateY(-50%);", y),

                div {
                    class: "text-xs bg-primary rounded-r-md px-1 py-0.5 text-foreground shrink-0",
                    "{time}"
                }

                div {
                    class: "h-[2px] flex-1 bg-primary/80 rounded-md"
                }
            }
        }
    } else {
        rsx! { "" }
    }
}