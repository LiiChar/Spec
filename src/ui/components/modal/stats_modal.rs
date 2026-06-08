use dioxus::prelude::*;

use crate::{core::EventModel, lib::{convert_ts_to_local_date, event_stats, format_duration_short}};
use dioxus_free_icons::icons::ld_icons::{LdX, LdPencil, LdTrash};
use dioxus_free_icons::Icon;


#[derive(Props, PartialEq, Clone)]
pub struct StatsModalProps {
  pub events: Vec<EventModel>,
  #[props(default = Signal::new(false))]
  pub visible: WriteSignal<bool>,
  #[props(default = Callback::new(|_| ()))]
  pub on_close: Callback<()>,
}

#[component]
pub fn StatsModal(props: StatsModalProps) -> Element {
    let evts = props.events.clone();
    if !(*props.visible.read()) {
        return rsx! { "" };
    }

    let stats = use_memo(move || {
      if props.events.is_empty() {
          None
      } else {
          Some(event_stats(props.events.clone()))
      }
    });

    let first_event = evts.first();
    let last_event = evts.last();

    let formatted_start = first_event
        .map(|e| {
            convert_ts_to_local_date(e.timestamp as u64)
                .format("%H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "--:--:--".to_string());

    let formatted_end = last_event
        .map(|e| {
            convert_ts_to_local_date(e.timestamp + e.duration as u64)
                .format("%H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "--:--:--".to_string());

    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 flex p-4 items-center justify-center z-[200]",

            onclick: move |_| {
                props.on_close.call(());
            },

            div {
                class: "bg-background p-6 rounded-lg shadow-lg max-w-md w-full relative h-full oveflow-auto",

                onclick: move |evt| evt.stop_propagation(),

                button {
                    class: "absolute top-2 right-2 hover:bg-destructive rounded-lg p-1 transition-colors",
                    onclick: move |_| props.on_close.call(()),

                    Icon { icon: LdX }
                }

                div {
                    class: "flex items-center gap-2 mb-1 justify-between border-b border-border/40 pb-1",

                    div {
                        class: "text-xs text-muted-foreground/60 flex gap-2 items-center",

                        span { "{formatted_start}" }
                        span { "-" }
                        span { "{formatted_end}" }
                    }
                }

                if evts.is_empty() {
                    div {
                        class: "py-10 text-center text-muted-foreground",

                        div {
                            class: "text-lg font-medium",
                            "Нет событий"
                        }

                        div {
                            class: "text-sm opacity-70 mt-1",
                            "За выбранный период данные отсутствуют"
                        }
                    }
                } else {
                    div {
                        class: "mt-4",

                        {
                            let s = stats().unwrap();

                            let format_active_percent =
                                format!("{:.1}%", s.active_percent);
                            let format_idle_percent =
                                format!("{:.1}%", s.idle_percent);

                            rsx! {
                                div {

                                    div { class: "grid grid-cols-2 gap-2 text-sm",

                                        div { class: "rounded-md border border-border/40 p-2",
                                            div { class: "text-xs opacity-70", "Общее время" }
                                            div { class: "font-medium",
                                                "{format_duration_short(s.total_time)}"
                                            }
                                        }

                                        div { class: "rounded-md border border-border/40 p-2",
                                            div { class: "text-xs opacity-70", "Активность" }
                                            div { class: "font-medium",
                                                "{format_duration_short(s.active_time)} ({format_active_percent})"
                                            }
                                        }

                                        div { class: "rounded-md border border-border/40 p-2",
                                            div { class: "text-xs opacity-70", "Простой" }
                                            div { class: "font-medium",
                                                "{format_duration_short(s.idle_time)} ({format_idle_percent})"
                                            }
                                        }

                                        div { class: "rounded-md border border-border/40 p-2",
                                            div { class: "text-xs opacity-70", "Событий" }
                                            div { class: "font-medium", "{s.num_events}" }
                                        }

                                        div { class: "rounded-md border border-border/40 p-2",
                                            div { class: "text-xs opacity-70", "Приложений" }
                                            div { class: "font-medium", "{s.num_apps}" }
                                        }

                                        div { class: "rounded-md border border-border/40 p-2",
                                            div { class: "text-xs opacity-70", "Средняя длительность" }
                                            div { class: "font-medium",
                                                "{format_duration_short(s.avg_event_duration)}"
                                            }
                                        }
                                    }

                                    if let Some(app) = &s.most_used_app {
                                        {
                                            let act_per =
                                                format!("{:.1}%", app.active_percent);

                                            rsx! {
                                                div {
                                                    class: "rounded-md border border-border/40 p-3 my-2",

                                                    div {
                                                        class: "text-xs opacity-70 mb-1",
                                                        "Самое используемое приложение"
                                                    }

                                                    div {
                                                        class: "font-medium",
                                                        "{app.name}"
                                                    }

                                                    div {
                                                        class: "text-sm opacity-80",
                                                        "{format_duration_short(app.total_time)} · {act_per} активного времени"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    if !s.app_list.is_empty() {
                                        div { class: "flex flex-col gap-1",

                                            for app in s.app_list.iter().take(5) {
                                                div {
                                                    class: "rounded-md border border-border/40 p-2",

                                                    div {
                                                        class: "flex justify-between gap-2",

                                                        div {
                                                            class: "truncate text-sm font-medium",
                                                            "{app.name}"
                                                        }

                                                        div {
                                                            class: "text-xs opacity-70 whitespace-nowrap",
                                                            "{format_duration_short(app.total_time)}"
                                                        }
                                                    }

                                                    div {
                                                        class: "mt-1 h-1.5 rounded-full bg-muted overflow-hidden",

                                                        div {
                                                            class: "h-full rounded-full bg-primary",
                                                            style: format!(
                                                                "width: {:.2}%",
                                                                app.active_percent
                                                            ),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}