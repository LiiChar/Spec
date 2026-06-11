use chrono::Local;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBarChart, LdX};
use dioxus_free_icons::Icon;

use crate::{
    lib::{convert_ts_to_local_date, y_to_timestamp, Segment},
    ui::{
        components::modal::stats_modal::StatsModal,
        context::{use_app, use_settings},
    },
};

#[derive(Props, PartialEq, Clone)]
pub struct TimelineSelectProps {
    pub segments: Vec<Segment>,
    pub selected_hour: Signal<Option<u32>>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DragState {
    start_y: f64,
    rect_top: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Selection {
    start: f64,
    end: f64,
}

#[component]
pub fn TimelineSelect(props: TimelineSelectProps) -> Element {
    let app = use_app();
    let settings = use_settings();

    let mut visible_stats = use_signal(|| false);
    let mut node_element = use_signal(|| None);
    let mut drag_state = use_signal(|| None::<DragState>);

    let mut selection = use_signal(|| None::<Selection>);
    let mut selected_events = use_signal(Vec::new);

    let mut pending_mouse_y = use_signal(|| None::<f64>);
    let mut raf_scheduled = use_signal(|| false);
    

    let day_start = use_memo(move || {
        app.day
            .read()
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis() as u64
    });

    let segments = props.segments.clone();

    // Memo: start & end timestamps, formatted start & end
    let time_info = use_memo(move || {
        let Some(sel) = selection() else {
            return None;
        };

        let settings = settings.settings.read();

        let start_ts = y_to_timestamp(
            sel.start,
            &segments,
            (props.selected_hour)(),
            day_start(),
            settings.segment_height,
            settings.selected_segment_height,
        );
        let end_ts = y_to_timestamp(
            sel.end,
            &segments,
            (props.selected_hour)(),
            day_start(),
            settings.segment_height,
            settings.selected_segment_height,
        );

        let formatted_start = convert_ts_to_local_date(start_ts)
            .format("%H:%M:%S")
            .to_string();
        let formatted_end = convert_ts_to_local_date(end_ts)
            .format("%H:%M:%S")
            .to_string();

        Some((start_ts, end_ts, formatted_start, formatted_end))
    });

    // Memo: top and height of the selection overlay
    let (top, height) = use_memo(move || {
        selection()
            .map(|s| (s.start.min(s.end), (s.end - s.start).abs()))
            .unwrap_or((0.0, 0.0))
    })();

    // Memo: display strings
    let formatted_start = time_info()
        .as_ref()
        .map(|v| v.2.clone())
        .unwrap_or_default();
    let formatted_end = time_info()
        .as_ref()
        .map(|v| v.3.clone())
        .unwrap_or_default();

    rsx! {
        StatsModal {
            visible: visible_stats,
            on_close: move |_| {
                visible_stats.set(false);
                selection.set(None);
            },
            events: selected_events(),
            start_ts: time_info().map(|v| v.0 as i64).unwrap_or_default(),
            end_ts: time_info().map(|v| v.1 as i64).unwrap_or_default(),
        }

        div {
            class: "absolute top-0 left-0 h-full w-full z-2",
            style: format!("z-index: {};", 
                match selection() {
                    Some(_) => "5",
                    None => "2",
                }
            ),
            onmounted: move |cx| {
                node_element.set(Some(cx.data()));
            },

            onpointerdown: move |evt| {
                evt.stop_propagation();

                let node = node_element.read().clone();
                spawn(async move {
                    if let Some(node) = node {
                        if let Ok(rect) = node.get_client_rect().await {
                            let y = evt.client_coordinates().y as f64 - rect.origin.y;

                            drag_state.set(Some(DragState {
                                start_y: y,
                                rect_top: rect.origin.y,
                            }));

                            selection.set(Some(Selection { start: y, end: y }));
                        }
                    }
                });
            },

            onpointermove: move |evt| {
                let Some(state) = *drag_state.read() else {
                    return;
                };

                // Always store the latest mouse y
                pending_mouse_y.set(Some(evt.client_coordinates().y as f64));

                // Throttle to one update per animation frame
                if !raf_scheduled() {
                    raf_scheduled.set(true);
                    spawn(async move {
                        if let Some(mouse_y) = pending_mouse_y() {
                            let local_y = mouse_y - state.rect_top;

                            let new_sel = Selection {
                                start: state.start_y.min(local_y),
                                end: state.start_y.max(local_y),
                            };

                            // Only update if selection actually changed
                            if selection() != Some(new_sel) {
                                selection.set(Some(new_sel));
                            }
                        }

                        raf_scheduled.set(false);
                    });
                }
            },

            onpointerup: move |_| {
                drag_state.set(None);

                let Some(sel) = selection() else {
                    return;
                };

                // Treat short drags as a click (no selection)
                if (sel.end - sel.start).abs() < 1.0 {
                    selection.set(None);
                    return;
                }

                let settings_read = settings.settings.read();

                let start_ts = y_to_timestamp(
                    sel.start,
                    &props.segments,
                    (props.selected_hour)(),
                    day_start(),
                    settings_read.segment_height,
                    settings_read.selected_segment_height,
                );
                let end_ts = y_to_timestamp(
                    sel.end,
                    &props.segments,
                    (props.selected_hour)(),
                    day_start(),
                    settings_read.segment_height,
                    settings_read.selected_segment_height,
                );

                let selection_start = start_ts.min(end_ts);
                let selection_end = start_ts.max(end_ts);

                let mut s_events = Vec::new();
                for event in app.events.read().iter() {
                    let event_start = event.timestamp;
                    let event_end = event.timestamp + event.duration;
                    if event_start < selection_end && event_end > selection_start {
                        s_events.push(event.clone());
                    }
                }
                s_events.sort_by_key(|e| e.timestamp);
                selected_events.set(s_events);
            },

            div {
                class: "absolute w-full left-0 bg-primary/20 z-50",
                style: format!("top:{top}px;height:{height}px;"),
                {
                    if let Some(sel) = selection() {
                        if sel.start != sel.end {
                            rsx! {
                                div {
                                    class: "absolute right-1 top-1 flex gap-0.5 text-sm",
                                    if drag_state().is_none() {
                                        button {
                                            class: "p-0.5! glass rounded-full",
                                            onpointerdown: |evt| evt.stop_propagation(),
                                            onclick: move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                evt.prevent_default();
                                                visible_stats.set(true);
                                            },
                                            Icon { icon: LdBarChart, width: 12, height: 12 }
                                        }
                                        button {
                                            class: "p-0.5! glass rounded-full",
                                            onpointerdown: |evt| evt.stop_propagation(),
                                            onclick: move |evt: Event<MouseData>| {
                                                evt.stop_propagation();
                                                evt.prevent_default();
                                                selection.set(None);
                                            },
                                            Icon { icon: LdX, width: 12, height: 12 }
                                        }
                                    } else {
                                        div {
                                            class: "flex gap-1 text-xs opacity-60 pointer-events-none select-none",
                                            div { "{formatted_start}" }
                                            div { "-" }
                                            div { "{formatted_end}" }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                ""
                            }
                        }
                    } else {
                        rsx! {
                            ""
                        }
                    }
                }
            }
        }
    }
}