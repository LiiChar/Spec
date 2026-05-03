use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

use crate::{
    core::{with_database, with_database_mut, EventModel, JobModel},
    ui::{AppProvider, SettingsProvider, ToasterProvider, Switch, Theme, Button},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsExportData {
    events: Vec<EventModel>,
    jobs: Vec<JobModel>,
}

fn export_data_to_json() -> Result<String, String> {
    let events = with_database(|db| db.get_all_events()).map_err(|err| err.to_string())?;
    let jobs = with_database(|db| db.get_jobs()).map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&SettingsExportData { events, jobs })
        .map_err(|err| err.to_string())
}

fn import_data_from_json(raw: &str) -> Result<(usize, usize), String> {
    let data: SettingsExportData = serde_json::from_str(raw).map_err(|err| err.to_string())?;

    with_database_mut(|db| {
        db.insert_events(&data.events)
            .map_err(|err| err.to_string())?;

        for job in &data.jobs {
            db.insert_jobs(job).map_err(|err| err.to_string())?;
        }

        Ok((data.events.len(), data.jobs.len()))
    })
}

fn pick_import_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import Spec data")
        .add_filter("JSON", &["json"])
        .pick_file()
}

fn pick_export_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export Spec data")
        .set_file_name("spec-data.json")
        .add_filter("JSON", &["json"])
        .save_file()
}

#[component]
pub fn SettingsPage() -> Element {
    let mut app = use_context::<AppProvider>();
    let settings = use_context::<SettingsProvider>();
    let mut toast = use_context::<ToasterProvider>();

    let mut theme = settings.theme;
    let mut enable_notifications = settings.enable_notifications;
    let notification_delay_ms = settings.notification_delay_ms;
    let tracker_report_interval_ms = settings.tracker_report_interval_ms;
    let db_flush_interval_ms = settings.db_flush_interval_ms;
    let mut event_limit = settings.event_limit;
    let mut compact_timeline = settings.compact_timeline;
    let mut show_idle_events = settings.show_idle_events;
    let mut auto_start_tracking = settings.auto_start_tracking;

    let mut selected_import_file = use_signal(String::new);
    let mut selected_export_file = use_signal(String::new);
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
                p { class: "text-sm text-foreground/60", "Параметры интерфейса, уведомлений, трекера и переноса данных." }
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
                            option { value: "dark", "Темная" }
                            option { value: "light", "Светлая" }
                        }
                    }

                    SettingsSwitch {
                        title: "Компактная таймлиния".to_string(),
                        hint: "Уплотняет отображение коротких событий.".to_string(),
                        checked: compact_timeline(),
                        onclick: move |_| compact_timeline.set(!compact_timeline()),
                    }

                    SettingsSwitch {
                        title: "Показывать idle".to_string(),
                        hint: "Оставляет периоды простоя в событиях.".to_string(),
                        checked: show_idle_events(),
                        onclick: move |_| show_idle_events.set(!show_idle_events()),
                    }

                    SettingsSwitch {
                        title: "Автостарт трекинга".to_string(),
                        hint: "Флаг для запуска трекера вместе с приложением.".to_string(),
                        checked: auto_start_tracking(),
                        onclick: move |_| auto_start_tracking.set(!auto_start_tracking()),
                    }
                }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                div {
                    class: "flex justify-between",
                    h2 { class: "mb-4 text-base font-semibold text-foreground", "Уведомления и сбор данных" }
                    Button {
                        onclick: move |_| {
                            toast.info("Тест".to_string(), Some("Тестовое описание".to_string()), None)
                        },
                        "Уведомление"
                    }
                }

                div { class: "grid gap-4 md:grid-cols-2",
                    SettingsSwitch {
                        title: "Уведомления".to_string(),
                        hint: "Разрешить отложенную отправку уведомлений.".to_string(),
                        checked: enable_notifications(),
                        onclick: move |_| enable_notifications.set(!enable_notifications()),
                    }

                    NumberSetting {
                        title: "Delay уведомлений, мс".to_string(),
                        value: notification_delay_ms().to_string(),
                        min: "0".to_string(),
                        oninput: update_u64(notification_delay_ms, 1_500),
                    }

                    NumberSetting {
                        title: "Интервал отчета трекера, мс".to_string(),
                        value: tracker_report_interval_ms().to_string(),
                        min: "250".to_string(),
                        oninput: update_u64(tracker_report_interval_ms, 5_000),
                    }

                    NumberSetting {
                        title: "Интервал записи в БД, мс".to_string(),
                        value: db_flush_interval_ms().to_string(),
                        min: "100".to_string(),
                        oninput: update_u64(db_flush_interval_ms, 750),
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
                    div {
                        h2 { class: "text-base font-semibold text-foreground", "Импорт и экспорт" }
                        p { class: "mt-1 text-xs text-foreground/55", "Формат файла: JSON с полями events и jobs." }
                    }

                    div { class: "flex flex-wrap gap-2",
                        button {
    class: "rounded-md border border-border/40 bg-background px-3 py-2 text-sm text-foreground hover:bg-foreground/5 disabled:opacity-50",
    disabled: is_busy(),
    onclick: move |_| {
        spawn(async move {
            let Some(path) = pick_export_file() else {
                return;
            };

            selected_export_file.set(path.display().to_string());
            is_busy.set(true);
            status.set("Готовлю экспорт...".to_string());

            let result = tokio::task::spawn_blocking(move || {
                let json = export_data_to_json()?;
                fs::write(&path, json).map_err(|err| err.to_string())?;
                Ok::<_, String>(path)
            }).await;

            match result {
                Ok(Ok(path)) => {
                    selected_export_file.set(path.display().to_string());
                    status.set(format!("Экспорт сохранен: {}", path.display()));
                }
                Ok(Err(err)) => {
                    status.set(format!("Ошибка экспорта: {err}"));
                }
                Err(err) => {
                    status.set(format!("Ошибка задачи экспорта: {err}"));
                }
            }

            is_busy.set(false);
        });
    },
    "Экспорт в файл"
}

                        button {
    class: "rounded-md border border-primary/40 bg-primary/20 px-3 py-2 text-sm text-foreground hover:bg-primary/30 disabled:opacity-50",
    disabled: is_busy(),
    onclick: move |_| {
        spawn(async move {
            let Some(path) = pick_import_file() else {
                return;
            };

            selected_import_file.set(path.display().to_string());
            is_busy.set(true);
            status.set("Импортирую данные...".to_string());

            let result = tokio::task::spawn_blocking(move || {
                let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
                let counts = import_data_from_json(&raw)?;
                Ok::<_, String>((path, counts))
            }).await;

            match result {
                Ok(Ok((path, (event_count, job_count)))) => {
                    let refreshed_events =
                        with_database(|db| db.get_all_events()).unwrap_or_default();
                    let refreshed_jobs =
                        with_database(|db| db.get_jobs()).unwrap_or_default();

                    app.events.set(refreshed_events);
                    app.jobs.set(refreshed_jobs);

                    selected_import_file.set(path.display().to_string());
                    status.set(format!(
                        "Импортировано: events {event_count}, jobs {job_count}."
                    ));
                }
                Ok(Err(err)) => {
                    status.set(format!("Ошибка импорта: {err}"));
                }
                Err(err) => {
                    status.set(format!("Ошибка задачи импорта: {err}"));
                }
            }

            is_busy.set(false);
        });
    },
    "Импорт из файла"
}
                    }
                }

                div { class: "grid gap-3 lg:grid-cols-2",
                    FileStatus {
                        title: "Последний экспорт".to_string(),
                        path: selected_export_file(),
                    }

                    FileStatus {
                        title: "Последний импорт".to_string(),
                        path: selected_import_file(),
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

#[component]
fn SettingsSwitch(
    title: String,
    hint: String,
    checked: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-3 rounded-md border border-border/30 p-3",
            div {
                div { class: "text-sm font-medium text-foreground", "{title}" }
                div { class: "text-xs text-foreground/55", "{hint}" }
            }
            Switch { checked, onclick }
        }
    }
}

#[component]
fn NumberSetting(
    title: String,
    value: String,
    min: String,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        label { class: "flex flex-col gap-2 text-sm text-foreground/70",
            "{title}"
            input {
                r#type: "number",
                min: "{min}",
                class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                value: "{value}",
                oninput,
            }
        }
    }
}

#[component]
fn FileStatus(title: String, path: String) -> Element {
    rsx! {
        div { class: "rounded-md border border-border/30 bg-foreground/5 p-3",
            div { class: "text-xs text-foreground/55", "{title}" }
            div { class: "mt-1 truncate text-sm text-foreground",
                if path.is_empty() {
                    "Файл еще не выбран"
                } else {
                    "{path}"
                }
            }
        }
    }
}
