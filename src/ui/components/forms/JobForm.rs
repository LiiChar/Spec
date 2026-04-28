use chrono::{DateTime, Local, Timelike};
use dioxus::prelude::*;

use crate::{
    config::DATABASE_PATH,
    core::{JobModel, WindowsDatabase},
    lib::convert_ts_to_local_date,
    ui::{ColorPicker, EventsCalendar, TimeInput },
};

#[derive(Props, PartialEq, Clone)]
pub struct JobFormProps {
    pub start_ts: i64,
    pub end_ts: i64,
    pub on_save: Callback<JobModel>,
    pub on_cancel: Callback<()>,
}

#[component]
pub fn JobForm(props: JobFormProps) -> Element {
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut cron = use_signal(|| String::new());
    let mut color = use_signal(|| "#3b82f6".to_string());
    let mut start_date =
        use_signal(|| convert_ts_to_local_date(props.start_ts.try_into().unwrap()));
    let mut end_date = use_signal(|| convert_ts_to_local_date(props.end_ts.try_into().unwrap()));
    let mut start_time = use_signal(|| (props.start_ts / 1000 / 60 / 60) as u32);
    let mut end_time = use_signal(|| (props.end_ts) as u32);

    let colors = [
        "#3b82f6", // blue-500
        "#22c55e", // green-500
        "#eab308", // yellow-500
        "#ef4444", // red-500
        "#a855f7", // purple-500
        "#ec4899", // pink-500
        "#6366f1", // indigo-500
        "#14b8a6", // teal-500
    ];

    rsx! {
        h2 {
            class: "text-lg font-semibold mb-4",
            "Создание задачи"
        }

        div {
            class: "space-y-4 overflow-x-hidden",
            div {
                class: "flex flex-col gap-1",
                label {
                    class: "text-sm font-medium",
                    "Название*"
                }
                input {
                    class: "w-full px-3 py-2 border border-border rounded-md bg-background",
                    value: "{name}",
                    oninput: move |e| name.set(e.value()),
                    placeholder: "Введите название задачи"
                }
            }

            // div {
            //     class: "flex flex-col gap-1",
            //     label {
            //         class: "text-sm font-medium",
            //         "Описание"
            //     }
            //     textarea {
            //         class: "w-full px-3 py-2 border border-border rounded-md bg-background resize-none",
            //         rows: "3",
            //         value: "{description}",
            //         oninput: move |e| description.set(e.value()),
            //         placeholder: "Введите описание (необязательно)"
            //     }
            // }

            // EventsCalendar {  }

            div {
                class: "flex gap-2",
                div {
                    class: "flex-1",
                    label {
                        class: "text-sm font-medium",
                        "Начало"
                    }
                    div {
                        class: "px-3 py-2 border border-border rounded-md bg-muted",
                        TimeInput { value: start_time}
                    }
                }
                div {
                    class: "flex-1",
                    label {
                        class: "text-sm font-medium",
                        "Конец"
                    }
                    div {
                        class: "px-3 py-2 border border-border rounded-md bg-muted",
                        TimeInput { value: end_time }
                    }
                }
            }

            div {
                class: "flex gap-2",
                div {
                    class: "flex flex-col gap-1",
                    label {
                        class: "text-sm font-medium",
                        "Cron"
                    }
                    input {
                        class: "w-full px-3 py-2 border border-border rounded-md bg-background",
                        value: "{cron}",
                        oninput: move |e| cron.set(e.value()),
                        placeholder: "0 9-17 * * 1-5"
                    }
                }

                div {
                    class: "flex flex-col gap-1",
                    label {
                        class: "text-sm font-medium",
                        "Цвет"
                    }
                    ColorPicker {
                        onselect: move |c: String| color.set(c),
                        color: color,
                        // colors: colors.iter().map(|c| String::from(*c)).collect(),
                    }
                }

            }
        }

        div {
            class: "flex gap-2 mt-6 justify-end",
            button {
                class: "px-4 py-2 border border-border rounded-md hover:bg-muted",
                onclick: move |_| props.on_cancel.call(()),
                "Отмена"
            }
            button {
                class: "px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90",
                onclick: move |_| {
                    if name().trim().is_empty() {
                        return;
                    }

                    let job = JobModel::new(
                        name().trim().to_string(),
                        props.start_ts,
                        props.end_ts,
                        Vec::new(),
                    );

                    let mut job = job;
                    if !description().trim().is_empty() {
                        job.description = Some(description().trim().to_string());
                    }
                    if !cron().trim().is_empty() {
                        job.cron = Some(cron().trim().to_string());
                    }
                    job.color = color();

                    // Save job to database
                    let job_clone = job.clone();
                    spawn(async move {
                        let result = tokio::task::spawn_blocking(move || {
                            let db = WindowsDatabase::new(DATABASE_PATH);
                            db.save_job(&job_clone)
                        })
                        .await;

                        match result {
                            Ok(Ok(job_id)) => {
                                println!("Job saved with id: {}", job_id);
                            }
                            Ok(Err(e)) => {
                                println!("Failed to save job: {:?}", e);
                            }
                            Err(e) => {
                                println!("Task error: {:?}", e);
                            }
                        }
                    });

                    props.on_save.call(job);
                },
                "Сохранить"
            }
        }
    }
}
