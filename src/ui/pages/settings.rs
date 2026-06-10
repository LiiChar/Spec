use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, rc::Rc};

use crate::{
    core::{
        EventModel, GoalModel, JobModel, with_database, with_database_mut
    },
    ui::{button::Button, Language, select::Select, select::SelectContent, select::SelectItem, select::SelectTrigger, switch::Switch, Theme, use_app, use_settings, use_toast, range::Range},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsExportData {
    events: Vec<EventModel>,
    jobs: Vec<JobModel>,
    #[serde(default)]
    goals: Vec<GoalModel>,
}

fn export_data_to_json() -> Result<String, String> {
    let events = with_database(|db| db.get_all_events()).map_err(|err| err.to_string())?;
    let jobs = with_database(|db| db.get_jobs()).map_err(|err| err.to_string())?;
    let goals = with_database(|db| db.get_goals()).map_err(|err| err.to_string())?;

    serde_json::to_string_pretty(&SettingsExportData { events, jobs, goals })
        .map_err(|err| err.to_string())
}

fn import_data_from_json(raw: &str) -> Result<(usize, usize, usize), String> {
    let data: SettingsExportData = serde_json::from_str(raw).map_err(|err| err.to_string())?;

    with_database_mut(|db| {
        db.insert_events(&data.events)
            .map_err(|err| err.to_string())?;

        for job in &data.jobs {
            db.insert_jobs(job).map_err(|err| err.to_string())?;
        }

        for goal in &data.goals {
            db.insert_goal(goal).map_err(|err| err.to_string())?;
        }

        Ok((data.events.len(), data.jobs.len(), data.goals.len()))
    })
}

fn pick_import_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Import Spexe data")
        .add_filter("JSON", &["json"])
        .pick_file()
}

fn pick_export_file() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Export Spexe data")
        .set_file_name("spec-data.json")
        .add_filter("JSON", &["json"])
        .save_file()
}

#[component]
pub fn SettingsPage() -> Element {
    let mut app = use_app();
    let settings_rc = Rc::new(use_settings());
    let settings = settings_rc.settings.read().clone();

    let mut toast = use_toast();
    let info: Callback<(String, Option<String>)> = Callback::new(move |(title, description)| toast.clone().info(title, description, None));

    let mut selected_import_file = use_signal(String::new);
    let mut selected_export_file = use_signal(String::new);
    let mut is_busy = use_signal(|| false);

    rsx! {
        div { class: "mx-auto flex w-full max-w-5xl flex-col gap-4 p-2",
            div { class: "flex flex-col gap-1",
                h1 { class: "text-xl font-semibold text-foreground", "Настройки" }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                h2 { class: "mb-4 text-base font-semibold text-foreground", "Интерфейс" }

                div { class: "grid gap-4 md:grid-cols-2",
                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Тема"
                        {
                            let st = settings_rc.clone();
                            rsx! {
                                Select {
                                    onchange: move |value: String| {
                                        let mut c = (*st).clone();
                                        c.set_theme(Theme::from_str(&value).unwrap_or_default());
                                    },
                                    value: "{settings.theme.as_str()}",
                                    SelectTrigger {}
                                    SelectContent {
                                        SelectItem { value: "light", title: "Светлая", "Светлая" }
                                        SelectItem { value: "dark", title: "Темная", "Темная" }
                                    }
                                }
                            }
                        }
                    
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Язык"
                        {
                            let st = settings_rc.clone();
                            rsx! {
                                Select {
                                    onchange: move |value: String| {
                                        let mut c = (*st).clone();
                                        c.set_language(Language::from_str(&value).unwrap_or_default());
                                    },
                                    value: "{settings.language.as_str()}",
                                    SelectTrigger {}
                                    SelectContent {
                                        SelectItem { value: "english", title: "English", "English" }
                                        SelectItem { value: "russian", title: "Русский", "Русский" }
                                    }
                                }
                            }
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Тип отображения названий событий"
                        {
                            let st = settings_rc.clone();
                            rsx! {
                                Select {
                                    onchange: move |value: String| {
                                        let mut c = (*st).clone();
                                        c.set_type_label(value);
                                    },
                                    value: "{settings.type_label}",
                                    SelectTrigger {}
                                    SelectContent {
                                        SelectItem { value: "full", title: "С иконкой и загаловком", "С иконкой и загаловком" }
                                        SelectItem { value: "text", title: "Без иконки", "Без иконки" }
                                        SelectItem { value: "title", title: "Только заголовок", "Только заголовок" }
                                        SelectItem { value: "label", title: "Только название приложения", "Только название приложения" }
                                    }
                                }
                            }
                        }
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Тип отображения тегов"
                        {
                            let st = settings_rc.clone();
                            rsx! {
                                Select {
                                    onchange: move |value: String| {
                                        let mut c = (*st).clone();
                                        c.set_type_tags(value);
                                    },
                                    value: "{settings.type_tags}",
                                    SelectTrigger {}
                                    SelectContent {
                                        SelectItem { value: "сircle", title: "Круги", "Круги" }
                                        SelectItem { value: "rectangle", title: "Столбцы", "Столбцы" }
                                    }
                                }
                            }
                        }
                    }

                    SettingsSwitch {
                        title: "Компактная таймлиния".to_string(),
                        hint: "Уплотняет отображение коротких событий."
                            .to_string(),
                        checked: settings.compact_timeline,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().compact_timeline;
                                c.set_compact_timeline(v);
                            }
                        },
                    }

                    SettingsSwitch {
                        title: "Показывать idle".to_string(),
                        hint: "Оставляет периоды простоя в событиях.".to_string(),
                        checked: settings.show_idle_events,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().show_idle_events;
                                c.set_show_idle_events(v);
                            }
                        },
                    }

                    SettingsSwitch {
                        title: "Автостарт трекинга".to_string(),
                        hint: "Флаг для запуска трекера вместе с приложением."
                            .to_string(),
                        checked: settings.auto_start_tracking,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().auto_start_tracking;
                                c.set_auto_start_tracking(v);
                            }
                        },
                    }

                    SettingsSwitch {
                        title: "Отображать теги".to_string(),
                        hint: ""
                            .to_string(),
                        checked: settings.show_tags,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().show_tags;
                                c.set_show_tags(v);
                            }
                        },
                    }

                    SettingsSwitch {
                        title: "Мягкие цвета событий".to_string(),
                        hint: ""
                            .to_string(),
                        checked: settings.soft_event,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().soft_event;
                                c.set_soft_event(v);
                            }
                        },
                    }

                    SettingsSwitch {
                        title: "Отображать линию текущего времени".to_string(),
                        hint: ""
                            .to_string(),
                        checked: settings.show_current_time_line,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().show_current_time_line;
                                c.set_show_current_time_line(v);
                            }
                        },
                    }

                    {
                        let st = settings_rc.clone();
                        
                        rsx! {
                            Range {
                                label: "Высота часа".to_string(),
                                value: settings.segment_height,
                                min: 1,
                                max: 2000,
                                step: 1,
                                on_input: move |v| {
                                    let mut c = (*st).clone();
                                    c.set_segment_height(v);
                                }
                            }
                        }
                    }

                    {
                        let st = settings_rc.clone();
                        
                        rsx! {
                            Range {
                                label: "Высота выбранного часа".to_string(),
                                value: settings.selected_segment_height,
                                min: 1,
                                max: 2000,
                                step: 1,
                                on_input: move |v| {
                                    let mut c = (*st).clone();
                                    c.set_selected_segment_height(v);
                                }
                            }
                        }
                    }

                    
                }
            }

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: " text-base font-semibold text-foreground",
                        "Уведомления и сбор данных"
                    }
                    Button {
                        class: "py-1!",
                        onclick: move |_| {
                            info((
                                "Тест".to_string(),
                                Some("Тестовое описание".to_string()),
                            ))
                        },
                        "Уведомление"
                    }
                }

                div { class: "grid gap-4 md:grid-cols-2",
                    SettingsSwitch {
                        title: "Уведомления".to_string(),
                        hint: "Разрешить отложенную отправку уведомлений."
                            .to_string(),
                        checked: settings.enable_notifications,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().enable_notifications;
                                c.set_notifications(v);
                            }
                        },
                    }

                    SettingsSwitch {
                        title: "База данных".to_string(),
                        hint: "Сохранять данные о статистике в базу данных."
                            .to_string(),
                        checked: settings.save_data,
                        onclick: {
                            let st = settings_rc.clone();
                            move |_| {
                                let mut c = (*st).clone();
                                let v = !c.settings.read().save_data;
                                c.set_save_data(v);
                            }
                        },
                    }

                    NumberSetting {
                        title: "Delay уведомлений, мс".to_string(),
                        value: format!("{}", settings.notification_delay_ms),
                        min: "0".to_string(),
                        oninput: {
                            let st = settings_rc.clone();
                            move |evt: FormEvent| {
                                let mut c = (*st).clone();
                                let v = evt.value().parse::<u64>().unwrap_or(1_500);
                                c.set_notification_delay(v);
                            }
                        },
                    }

                    NumberSetting {
                        title: "Интервал отчета трекера, мс".to_string(),
                        value: format!("{}", settings.report_interval),
                        min: "250".to_string(),
                        oninput: {
                            let st = settings_rc.clone();
                            move |evt: FormEvent| {
                                let mut c = (*st).clone();
                                let v = evt.value().parse::<u64>().unwrap_or(5_000);
                                c.set_tracker_interval(v);
                            }
                        },
                    }

                    NumberSetting {
                        title: "Интервал записи в БД, мс".to_string(),
                        value: format!("{}", settings.db_flush_interval_ms),
                        min: "100".to_string(),
                        oninput: {
                            let st = settings_rc.clone();
                            move |evt: FormEvent| {
                                let mut c = (*st).clone();
                                let v = evt.value().parse::<u64>().unwrap_or(750);
                                c.set_db_flush_interval(v);
                            }
                        },
                    }

                    label { class: "flex flex-col gap-2 text-sm text-foreground/70",
                        "Лимит событий в памяти"
                        input {
                            r#type: "number",
                            min: "1",
                            class: "h-10 rounded-md border border-border/40 bg-background px-3 text-foreground outline-none focus:border-primary",
                            value: "{settings.event_limit}",
                            oninput: {
                                let st = settings_rc.clone();
                                move |evt| {
                                    let mut c = (*st).clone();
                                    let value = evt.value().parse::<i64>().unwrap_or(1_000).max(1);
                                    c.set_event_limit(value);
                                }
                            },
                        }
                    }
                }
            }

            

            section { class: "rounded-md border border-border/40 bg-background/70 p-4",
                div { class: "mb-4 flex flex-wrap items-center justify-between gap-3",
                    div {
                        h2 { class: "text-base font-semibold text-foreground",
                            "Импорт и экспорт"
                        }
                        p { class: "mt-1 text-xs text-foreground/55",
                            "Формат файла: JSON с полями events, jobs и goals."
                        }
                    }

                    div { class: "flex flex-wrap gap-2",
                        button {
                            class: "rounded-md border border-border/40 bg-background px-3 py-2 text-sm text-foreground hover:bg-foreground/5 disabled:opacity-50",
                            disabled: is_busy(),
                            onclick: move |_| {
                                if let Some(path) = pick_export_file() {
                                    selected_export_file.set(path.display().to_string());
                                    is_busy.set(true);
                                    info(("Готовлю экспорт...".to_string(), None));

                                    match export_data_to_json()
                                        .and_then(|json| {
                                            fs::write(&path, json).map_err(|err| err.to_string())
                                        })
                                    {
                                        Ok(_) => {
                                            selected_export_file.set(path.display().to_string());
                                            info((
                                                format!("Экспорт сохранен: {}", path.display()),
                                                None,
                                            ));
                                        }
                                        Err(err) => {
                                            info((format!("Ошибка экспорта: {err}"), None));
                                        }
                                    }
                                    is_busy.set(false);
                                }
                            },
                            "Экспорт в файл"
                        }

                        button {
                            class: "rounded-md border border-primary/40 bg-primary/20 px-3 py-2 text-sm text-foreground hover:bg-primary/30 disabled:opacity-50",
                            disabled: is_busy(),
                            onclick: move |_| {
                                if let Some(path) = pick_import_file() {
                                    selected_import_file.set(path.display().to_string());
                                    is_busy.set(true);
                                    info(("Импортирую данные...".to_string(), None));

                                    let res: Result<(usize, usize, usize), String> = (|| {
                                        let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
                                        import_data_from_json(&raw)
                                    })();

                                    match res {
                                        Ok((event_count, job_count, goal_count)) => {
                                            let refreshed_events = with_database(|db| db.get_all_events())
                                                .unwrap_or_default();
                                            let refreshed_jobs = with_database(|db| db.get_jobs())
                                                .unwrap_or_default();
                                            app.events.set(refreshed_events);
                                            app.jobs.set(refreshed_jobs);
                                            selected_import_file.set(path.display().to_string());
                                            info((
                                                format!(
                                                    "Импортировано: events {event_count}, jobs {job_count}, goals {goal_count}.",
                                                ),
                                                None,
                                            ));
                                        }
                                        Err(err) => {
                                            info((format!("Ошибка импорта: {err}"), None));
                                        }
                                    }
                                    is_busy.set(false);
                                }
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
                if !hint.is_empty() {
                    div { class: "text-xs text-foreground/55", "{hint}" }
                }
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
