use std::collections::HashMap;

use dioxus::prelude::*;

use crate::{
    core::EventModel,
    lib::format_duration_short,
    ui::{
        AppProvider, ChartType, EventsCharts, EventsList, EventsStats, EventsTimelineView, JobForm,
        Tabs, TabsContent, TabsList, TabsTrigger, TabsVariant, TimelineOrientation,
    },
};

#[derive(PartialEq, Clone)]
pub enum ViewMode {
    Timeline,
    List,
    Stats,
}

#[component]
pub fn Events() -> Element {
    let context = use_context::<AppProvider>();
    let events = context.events;
    let jobs = context.jobs;
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
        div { class: "flex flex-col gap-4 h-full w-full -mt-4.5",
            Tabs {
                variant: TabsVariant::Rounded,
                value: "timeline".to_string(),
                TabsList {
                    class: "sticky top-2 z-100" ,
                    TabsTrigger { value: "timeline".to_string(), "График" }
                    TabsTrigger { value: "statistics".to_string(), "Статистика" }
                }

                TabsContent {
                    value: "timeline".to_string(),
                    div { class: "flex-1 flex flex-row gap-3 py-1 pt-2",

                        div { class: "flex-1 flex z-50",
                            EventsTimelineView {
                                events: events.clone(),
                                jobs: jobs.clone(),
                                orientation: TimelineOrientation::Vertical,
                                day: context.day,
                            }
                        }

                        div { class: "hidden  lg:flex w-72 flex-col gap-3 bg-secondary/40 rounded-lg border border-border/30 p-4 ",

                            div { class: "grid grid-cols-3 gap-2",

                                div { class: "rounded-md bg-background/50 px-2 py-2 text-center",
                                    div { class: "text-[10px] uppercase tracking-wide text-muted-foreground",
                                        "События"
                                    }
                                    div { class: "text-base font-semibold text-primary", "{event_count}" }
                                }

                                div { class: "rounded-md bg-background/50 px-2 py-2 text-center",
                                    div { class: "text-[10px] uppercase tracking-wide text-muted-foreground",
                                        "Приложения"
                                    }
                                    div { class: "text-base font-semibold text-primary", "{unique_apps}" }
                                }

                                div { class: "rounded-md bg-background/50 px-2 py-2 text-center",
                                    div { class: "text-[10px] uppercase tracking-wide text-muted-foreground",
                                        "Время"
                                    }
                                    div { class: "text-base font-semibold text-primary",
                                        "{format_duration_short(total_time)}"
                                    }
                                }
                            }

                            div { class: "flex flex-col gap-2 pt-2",

                                h3 { class: "text-xs font-semibold text-foreground uppercase tracking-wider",
                                    "Топ Приложений"
                                }

                                {top_apps.into_iter().map(|(app, duration)| rsx! {
                                    div { class: "flex flex-row items-center justify-between p-2 bg-background/30 rounded-md hover:bg-background/50 transition-colors cursor-default",

                                        span { class: "text-xs font-medium text-foreground truncate", "{app}" }

                                        span { class: "text-sm font-bold text-green-400 whitespace-nowrap ml-2",
                                            "{format_duration_short(duration)}"
                                        }
                                    }
                                })
                                }
                            }
                        }
                    }
                }

                TabsContent {
                    value: "statistics".to_string(),
                    div { class: "flex-1 flex flex-col pt-2", EventsStats {} }
                }
            }
        }
    }
}
