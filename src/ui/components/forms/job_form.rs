use chrono::{DateTime, Local, NaiveDate, TimeZone, Timelike};
use dioxus::prelude::*;

use crate::{
    core::{JobModel, TagModel},
    lib::convert_ts_to_local_date,
    ui::{ColorPicker, TimeInput},
};

#[derive(Props, PartialEq, Clone)]
pub struct JobFormProps {
    #[props(default = None)]
    pub job: Option<JobModel>,
    /// Календарный день для новой задачи (время задаётся отдельно).
    pub day: NaiveDate,
    /// Секунды от полуночи для начала (новая задача).
    pub start_ts: i64,
    /// Секунды от полуночи для конца (новая задача).
    pub end_ts: i64,
    pub on_save: Callback<JobModel>,
    pub on_cancel: Callback<()>,
}

fn ts_to_time(ts: i64) -> u32 {
    let dt = Local.timestamp_opt(ts, 0).single().unwrap();
    (dt.hour() * 3600 + dt.minute() * 60 + dt.second()) as u32
}

fn naive_day_start(d: NaiveDate) -> DateTime<Local> {
    let n = d.and_hms_opt(0, 0, 0).unwrap();
    Local
        .from_local_datetime(&n)
        .latest()
        .unwrap_or_else(|| Local::now())
}

fn combine(date: DateTime<Local>, time: u32) -> i64 {
    let dt = date
        .date_naive()
        .and_hms_opt((time / 3600) % 24, (time % 3600) / 60, time % 60)
        .unwrap();

    Local
        .from_local_datetime(&dt)
        .latest()
        .unwrap_or_else(|| Local::now())
        .timestamp()
}

#[component]
pub fn JobForm(props: JobFormProps) -> Element {
    let is_edit = props.job.is_some();

    let mut name = use_signal(|| {
        props
            .job
            .as_ref()
            .map(|j| j.name.clone())
            .unwrap_or_default()
    });
    let mut description = use_signal(|| {
        props
            .job
            .as_ref()
            .and_then(|j| j.description.clone())
            .unwrap_or_default()
    });
    let mut cron = use_signal(|| {
        props
            .job
            .as_ref()
            .and_then(|j| j.cron.clone())
            .unwrap_or_default()
    });
    let mut color = use_signal(|| {
        props
            .job
            .as_ref()
            .map(|j| j.color.clone())
            .unwrap_or("#3b82f6".to_string())
    });

    let day_anchor = props.day;
    let mut start_date = use_signal(|| {
        props.job.as_ref().map_or_else(
            || naive_day_start(day_anchor),
            |j| convert_ts_to_local_date((j.start_ts as i64 * 1000) as u64),
        )
    });
    let mut end_date = use_signal(|| {
        props.job.as_ref().map_or_else(
            || naive_day_start(day_anchor),
            |j| convert_ts_to_local_date((j.end_ts as i64 * 1000) as u64),
        )
    });

    let mut start_time = use_signal(|| {
        props
            .job
            .as_ref()
            .map(|j| ts_to_time(j.start_ts))
            .unwrap_or(props.start_ts.clamp(0, 86_399) as u32)
    });

    let mut end_time = use_signal(|| {
        props
            .job
            .as_ref()
            .map(|j| ts_to_time(j.end_ts))
            .unwrap_or(props.end_ts.clamp(0, 86_399) as u32)
    });

    let mut tags_line = use_signal(|| {
        props
            .job
            .as_ref()
            .map(|j| j.tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", "))
            .unwrap_or_default()
    });

    rsx! {
        h2 {
            class: "text-lg font-semibold mb-4",
            if is_edit { "Редактирование задачи" } else { "Создание задачи" }
        }

        div {
            class: "space-y-4",

            // NAME
            div {
                label { class: "text-sm font-medium", "Название*" }
                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                }
            }

            div {
                label { class: "text-sm font-medium", "Описание" }
                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",
                    value: "{description}",
                    oninput: move |e| description.set(e.value()),
                    placeholder: "Необязательно"
                }
            }

            // TIME
            div {
                class: "flex gap-2",

                div {
                    class: "flex-1",
                    label { "Начало" }
                    TimeInput { value: start_time }
                }

                div {
                    class: "flex-1",
                    label { "Конец" }
                    TimeInput { value: end_time }
                }
            }

            // CRON + COLOR
            div {
                class: "flex gap-2",

                input {
                    class: "flex-1 px-3 py-2 border rounded-md bg-background",
                    value: "{cron}",
                    oninput: move |e| cron.set(e.value()),
                    placeholder: "0 9-17 * * 1-5"
                }

                ColorPicker {
                    color: color,
                    onselect: move |c: String| {
                        color.set(c.clone());
                        
                        document::eval(format!("document.querySelector('.job-modal-refw').style.outline = '2px solid {}';", c).as_str());
                    },
                }
            }

            div {
                label { class: "text-sm font-medium", "Теги (через запятую)" }
                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",
                    value: "{tags_line}",
                    oninput: move |e| tags_line.set(e.value()),
                    placeholder: "код, веб, созвон"
                }
            }
        }

        div {
            class: "flex justify-end gap-2 mt-6",

            button {
                class: "px-4 py-2 border rounded-md",
                onclick: move |_| props.on_cancel.call(()),
                "Отмена"
            }

            button {
                class: "px-4 py-2 bg-primary text-white rounded-md",

                onclick: move |_| {
                    if name().trim().is_empty() {
                        return;
                    }

                    let start_ts = combine(start_date(), start_time());
                    let end_ts = combine(end_date(), end_time());

                    let mut job = props.job.clone().unwrap_or_else(|| {
                        JobModel::new(
                            name().trim().to_string(),
                            start_ts,
                            end_ts,
                            Vec::new(),
                        )
                    });

                    job.name = name().trim().to_string();
                    job.start_ts = start_ts;
                    job.end_ts = end_ts;
                    job.color = color();
                    job.description = if description().trim().is_empty() {
                        None
                    } else {
                        Some(description().trim().to_string())
                    };

                    if !cron().trim().is_empty() {
                        job.cron = Some(cron().trim().to_string());
                    }

                    job.tags = tags_line()
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|tag_name| TagModel::new(tag_name, None, "#94a3b8"))
                        .collect();

                    props.on_save.call(job);
                },

                if is_edit { "Сохранить" } else { "Создать" }
            }
        }
    }
}
