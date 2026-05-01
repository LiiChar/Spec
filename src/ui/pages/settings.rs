use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::{with_database, with_database_mut, EventModel, JobModel},
    ui::{AppProvider, SettingsProvider, Switch, Theme},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsExportData {
    events: Vec<EventModel>,
    jobs: Vec<JobModel>,
}

#[component]
pub fn SettingsPage() -> Element {
    let mut app = use_context::<AppProvider>();
    let settings = use_context::<SettingsProvider>();

    let mut theme = settings.theme;
    let mut enable_notifications = settings.enable_notifications;
    let mut notification_delay_ms = settings.notification_delay_ms;
    let mut tracker_report_interval_ms = settings.tracker_report_interval_ms;
    let mut db_flush_interval_ms = settings.db_flush_interval_ms;
    let mut event_limit = settings.event_limit;
    let mut compact_timeline = settings.compact_timeline;
    let mut show_idle_events = settings.show_idle_events;
    let mut auto_start_tracking = settings.auto_start_tracking;

    let mut export_text = use_signal(String::new);
    let mut import_text = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut is_busy = use_signal(|| false);

    let update_u64 = |mut signal: Signal<u64>, fallback: u64| {
        move |evt: FormEvent| {
            let value = evt.value().parse::<u64>().unwrap_or(fallback);
            signal.set(value);
        }
    };

    rsx! {
        div { class: "mx-auto flex w-full max-w-5xl flex-col gap-4 p-2",
            div { class: "flex flex-col gap-1",
                h1 { class: "text-xl font-semibold text-foreground", "Настройки" }
                p { class: "text-sm text-foreground/60", "Параметры интерфейса, уведомлений, трекера и перенос данных." }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                h2 { class: "mb-4 text-base font-semibold text-foreground", "Интерфейс" }

                div { class: "grid gap-4 md:grid-cols-2",
                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Тема"
                        select {
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            value: "{theme().as_str()}",
                            onchange: move |evt| {
                                match evt.value().as_str() {
                                    "light" => theme.set(Theme::Light),
                                    _ => theme.set(Theme::Dark),
                                }
                            },
                            option { value: "dark", "Тёмная" }
                            option { value: "light", "Светлая" }
                        }
                    }

                    div { class: "flex items-center justify-between gap-3 rounded-md border border-border/30 p-3",
                        div {
                            div { class: "text-sm font-medium text-foreground", "Компактная таймлиния" }
                            div { class: "text-xs text-foreground/55", "Уплотняет отображение коротких событий." }
                        }
                        Switch {
                            checked: compact_timeline(),
                            onclick: move |_| compact_timeline.set(!compact_timeline()),
                        }
                    }

                    div { class: "flex items-center justify-between gap-3 rounded-md border border-border/30 p-3",
                        div {
                            div { class: "text-sm font-medium text-foreground", "Показывать idle" }
                            div { class: "text-xs text-foreground/55", "Оставляет периоды простоя в событиях." }
                        }
                        Switch {
                            checked: show_idle_events(),
                            onclick: move |_| show_idle_events.set(!show_idle_events()),
                        }
                    }

                    div { class: "flex items-center justify-between gap-3 rounded-md border border-border/30 p-3",
                        div {
                            div { class: "text-sm font-medium text-foreground", "Автостарт трекинга" }
                            div { class: "text-xs text-foreground/55", "Флаг для запуска трекера вместе с приложением." }
                        }
                        Switch {
                            checked: auto_start_tracking(),
                            onclick: move |_| auto_start_tracking.set(!auto_start_tracking()),
                        }
                    }
                }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                h2 { class: "mb-4 text-base font-semibold text-foreground", "Уведомления и сбор данных" }

                div { class: "grid gap-4 md:grid-cols-2",
                    div { class: "flex items-center justify-between gap-3 rounded-md border border-border/30 p-3",
                        div {
                            div { class: "text-sm font-medium text-foreground", "Уведомления" }
                            div { class: "text-xs text-foreground/55", "Разрешить отложенную отправку уведомлений." }
                        }
                        Switch {
                            checked: enable_notifications(),
                            onclick: move |_| enable_notifications.set(!enable_notifications()),
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Delay уведомлений, мс"
                        input {
                            r#type: "number",
                            min: "0",
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            value: "{notification_delay_ms()}",
                            oninput: update_u64(notification_delay_ms, 1_500),
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Интервал отчёта трекера, мс"
                        input {
                            r#type: "number",
                            min: "250",
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            value: "{tracker_report_interval_ms()}",
                            oninput: update_u64(tracker_report_interval_ms, 5_000),
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Интервал записи в БД, мс"
                        input {
                            r#type: "number",
                            min: "100",
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            value: "{db_flush_interval_ms()}",
                            oninput: update_u64(db_flush_interval_ms, 750),
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Лимит событий в памяти"
                        input {
                            r#type: "number",
                            min: "1",
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            value: "{event_limit()}",
                            oninput: move |evt| {
                                let value = evt.value().parse::<i64>().unwrap_or(1_000).max(1);
                                event_limit.set(value);
                            },
                        }
                    }
                }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                div { class: "mb-4 flex flex-wrap items-center justify-between gap-3",
                    h2 { class: "text-base font-semibold text-foreground", "Импорт и экспорт" }
                    div { class: "flex flex-wrap gap-2",
                        button {
                            class: "rounded-md border border-border/40 bg-background px-3 py-2 text-sm text-foreground hover:bg-foreground/5 disabled:opacity-50",
                            disabled: is_busy(),
                            onclick: move |_| {
                                is_busy.set(true);
                                status.set("Готовлю экспорт...".to_string());
                                spawn(async move {
                                    let result = tokio::task::spawn_blocking(move || {
                                        let events = with_database(|db| db.get_all_events())
                                            .map_err(|err| err.to_string())?;
                                        let jobs = with_database(|db| db.get_jobs())
                                            .map_err(|err| err.to_string())?;
                                        serde_json::to_string_pretty(&SettingsExportData { events, jobs })
                                            .map_err(|err| err.to_string())
                                    }).await;

                                    match result {
                                        Ok(Ok(json)) => {
                                            export_text.set(json);
                                            status.set("Экспорт готов.".to_string());
                                        }
                                        Ok(Err(err)) => status.set(format!("Ошибка экспорта: {err}")),
                                        Err(err) => status.set(format!("Ошибка задачи экспорта: {err}")),
                                    }
                                    is_busy.set(false);
                                });
                            },
                            "Экспортировать"
                        }

                        button {
                            class: "rounded-md border border-primary/40 bg-primary/20 px-3 py-2 text-sm text-foreground hover:bg-primary/30 disabled:opacity-50",
                            disabled: is_busy() || import_text.read().trim().is_empty(),
                            onclick: move |_| {
                                let raw = import_text();
                                is_busy.set(true);
                                status.set("Импортирую данные...".to_string());
                                spawn(async move {
                                    let result = tokio::task::spawn_blocking(move || {
                                        let data: SettingsExportData = serde_json::from_str(&raw)
                                            .map_err(|err| err.to_string())?;

                                        with_database_mut(|db| {
                                            db.insert_events(&data.events).map_err(|err| err.to_string())?;
                                            for job in &data.jobs {
                                                db.insert_jobs(job).map_err(|err| err.to_string())?;
                                            }
                                            Ok::<_, String>((data.events.len(), data.jobs.len()))
                                        })
                                    }).await;

                                    match result {
                                        Ok(Ok((event_count, job_count))) => {
                                            let refreshed_events = with_database(|db| db.get_all_events()).unwrap_or_default();
                                            let refreshed_jobs = with_database(|db| db.get_jobs()).unwrap_or_default();
                                            app.events.set(refreshed_events);
                                            app.jobs.set(refreshed_jobs);
                                            status.set(format!("Импортировано: events {event_count}, jobs {job_count}."));
                                        }
                                        Ok(Err(err)) => status.set(format!("Ошибка импорта: {err}")),
                                        Err(err) => status.set(format!("Ошибка задачи импорта: {err}")),
                                    }
                                    is_busy.set(false);
                                });
                            },
                            "Импортировать"
                        }
                    }
                }

                div { class: "grid gap-4 lg:grid-cols-2",
                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Экспортированный JSON"
                        textarea {
                            class: "min-h-72 w-full resize-y rounded-md border border-border/40 bg-background p-3 font-mono text-xs text-foreground outline-none focus:border-primary",
                            value: "{export_text()}",
                            readonly: true,
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "JSON для импорта"
                        textarea {
                            class: "min-h-72 w-full resize-y rounded-md border border-border/40 bg-background p-3 font-mono text-xs text-foreground outline-none focus:border-primary",
                            value: "{import_text()}",
                            oninput: move |evt| import_text.set(evt.value()),
                        }
                    }
                }

                if !status().is_empty() {
                    div { class: "mt-3 rounded-md border border-border/30 bg-foreground/5 px-3 py-2 text-sm text-foreground/70",
                        "{status}"
                    }
                }
            }
        }
    }
}
