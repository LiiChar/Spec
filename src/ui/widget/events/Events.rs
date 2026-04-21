use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{
    core::EventModel,
    lib::format_duration_short,
    ui::{AppContext, ChartType, EventsCharts, EventsList, EventsTimeline, TimelineOrientation},
};

#[derive(PartialEq, Clone)]
pub enum ViewMode {
    Timeline,
    List,
}

#[component]
pub fn Events() -> Element {
    let context = use_context::<AppContext>();
    let events = context.events;
    let mut view_mode: Signal<ViewMode> = use_signal(|| ViewMode::Timeline);

    let value = events.clone();
    let summary = use_memo(move || {
        let items = &value;
        let total_time: u64 = items.iter().map(|event| event.duration).sum();

        let mut app_times: HashMap<String, u64> = HashMap::new();
        for event in items.iter() {
            if let Some(window) = &event.window {
                *app_times.entry(window.process_name.clone()).or_default() += event.duration;
            }
        }

        let unique_apps = app_times.len();
        let mut top_apps: Vec<(String, u64)> = app_times.into_iter().collect();
        top_apps.sort_by(|a, b| b.1.cmp(&a.1));
        top_apps.truncate(10);

        (items.len(), total_time, unique_apps, top_apps)
    });

    let (event_count, total_time, unique_apps, top_apps) = summary();

    rsx! {
        div {
            class: "flex flex-col gap-0 h-full w-full",

            div {
                class: "mb-6 relative",

                div {
                    class: "fixed top-2 left-2 flex flex-row items-center justify-between rounded-lg bg-secondary/20 backdrop-blur-lg z-10 border border-border/30",

                    button {
                        class: format!(
                            "p-1 px-2 rounded-l-lg cursor-pointer {}",
                            if view_mode() == ViewMode::Timeline {
                                "bg-background"
                            } else {
                                ""
                            },
                        ),
                        onclick: move |_| view_mode.set(ViewMode::Timeline),
                        "График"
                    }

                    button {
                        class: format!(
                            "p-1 px-2 rounded-r-lg cursor-pointer {}",
                            if view_mode() == ViewMode::List {
                                "bg-background"
                            } else {
                                ""
                            },
                        ),
                        onclick: move |_| view_mode.set(ViewMode::List),
                        "Список"
                    }
                }
            }

            // EventsCharts {charts: vec![ChartType::Bar, ChartType::Timeline]}

            match view_mode() {
                ViewMode::Timeline => rsx! {
                    div {
                        class: "flex-1 flex flex-row gap-2 p-2",

                        div {
                            class: "flex-1 flex",
                            EventsTimeline {
                                events: events.clone(),
                                orientation: TimelineOrientation::Vertical,
                                day: context.day
                            }
                        }

                        div {
                            class: "hidden lg:flex w-80 flex-col gap-3 bg-zinc-900/30 rounded-lg border border-zinc-700/30 p-4",

                            div {
                                class: "grid grid-cols-3 gap-3",

                                div {
                                    class: "rounded-md bg-zinc-800/40 px-3 py-2",
                                    div { class: "text-[11px] uppercase tracking-wide text-gray-400", "События" }
                                    div { class: "text-lg font-bold text-cyan-400", "{event_count}" }
                                }

                                div {
                                    class: "rounded-md bg-zinc-800/40 px-3 py-2",
                                    div { class: "text-[11px] uppercase tracking-wide text-gray-400", "Приложения" }
                                    div { class: "text-lg font-bold text-purple-400", "{unique_apps}" }
                                }

                                div {
                                    class: "rounded-md bg-zinc-800/40 px-3 py-2",
                                    div { class: "text-[11px] uppercase tracking-wide text-gray-400", "Время" }
                                    div { class: "text-lg font-bold text-green-400", "{format_duration_short(total_time)}" }
                                }
                            }

                            div {
                                class: "flex flex-col gap-3 pt-1",

                                h3 {
                                    class: "text-sm font-bold text-cyan-400 uppercase tracking-widest",
                                    "Топ Приложений"
                                }

                                {top_apps.into_iter().map(|(app, duration)| rsx! {
                                    div {
                                        class: "flex flex-row items-center justify-between p-2 bg-zinc-800/40 rounded-md hover:bg-zinc-700/50 transition-colors",

                                        span {
                                            class: "text-sm font-semibold text-gray-300 truncate",
                                            "{app}"
                                        }

                                        span {
                                            class: "text-sm font-bold text-green-400 whitespace-nowrap ml-2",
                                            "{format_duration_short(duration)}"
                                        }
                                    }
                                })}
                            }
                        }
                    }
                },
                ViewMode::List => rsx! {
                    div {
                        class: "flex-1 flex flex-col p-2",
                        EventsList { events }
                    }
                }
            }
        }
    }
}
