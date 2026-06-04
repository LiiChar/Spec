use chrono::{DateTime, Local, NaiveDate, TimeZone};
use dioxus::prelude::*;

use crate::{
    core::{GoalModel, GoalOrder, TagModel},
    lib::convert_ts_to_local_date,
};

#[derive(Props, PartialEq, Clone)]
pub struct GoalFormProps {
    pub day: NaiveDate,
    #[props(default = None)]
    pub goal: Option<GoalModel>,
    pub on_save: Callback<GoalModel>,
    pub on_cancel: Callback<()>,
}

fn naive_day_start(d: NaiveDate) -> DateTime<Local> {
    let n = d.and_hms_opt(0, 0, 0).unwrap();
    Local
        .from_local_datetime(&n)
        .latest()
        .unwrap_or_else(|| Local::now())
}

fn day_end_timestamp(d: NaiveDate) -> i64 {
    let n = d.and_hms_opt(23, 59, 59).unwrap();
    Local
        .from_local_datetime(&n)
        .latest()
        .unwrap_or_else(|| Local::now())
        .timestamp()
}

fn day_start_timestamp(d: NaiveDate) -> i64 {
    naive_day_start(d).timestamp()
}

#[component]
pub fn GoalForm(props: GoalFormProps) -> Element {
    let is_edit = props.goal.is_some();
    let anchor = props.day;

    let mut name = use_signal(|| props.goal.as_ref().map(|g| g.name.clone()).unwrap_or_default());
    let mut description = use_signal(|| {
        props
            .goal
            .as_ref()
            .and_then(|g| g.description.clone())
            .unwrap_or_default()
    });
    let mut process =
        use_signal(|| props.goal.as_ref().map(|g| g.process.clone()).unwrap_or_default());
    let mut completed = use_signal(|| props.goal.as_ref().is_some_and(|g| g.completed));
    let mut ordering = use_signal(|| {
        props
            .goal
            .as_ref()
            .map(|g| g.ordering)
            .unwrap_or(GoalOrder::Equal)
    });

    let mut start_date = use_signal(|| {
        props.goal.as_ref().map_or_else(
            || naive_day_start(anchor),
            |g| convert_ts_to_local_date((g.start_period_ts as i64 * 1000) as u64),
        )
    });
    let mut end_date = use_signal(|| {
        props.goal.as_ref().map_or_else(
            || naive_day_start(anchor),
            |g| convert_ts_to_local_date((g.end_period_ts as i64 * 1000) as u64),
        )
    });

    let mut tags_line = use_signal(|| {
        props
            .goal
            .as_ref()
            .map(|g| {
                g.tags
                    .iter()
                    .map(|t| t.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    });

    rsx! {
        h2 { class: "text-lg font-semibold mb-4",
            if is_edit {
                "Редактирование цели"
            } else {
                "Новая цель"
            }
        }

        div { class: "space-y-4",
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
                    placeholder: "Необязательно",
                }
            }

            div {
                label { class: "text-sm font-medium", "Процесс / exe" }
                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",
                    value: "{process}",
                    oninput: move |e| process.set(e.value()),
                    placeholder: "chrome.exe, Code.exe...",
                }
            }

            label { class: "flex items-center gap-2 text-sm cursor-pointer",
                input {
                    r#type: "checkbox",
                    checked: completed(),
                    onclick: move |_| completed.set(!completed()),
                }
                "Выполнено"
            }

            label { class: "flex flex-col gap-1 text-sm",
                "Порог (ordering)"
                select {
                    class: "h-10 rounded-md border border-border/40 bg-background px-3",
                    value: "{ordering().as_str()}",
                    onchange: move |e| {
                        ordering
                            .set(
                                match e.value().as_str() {
                                    "<" => GoalOrder::Less,
                                    ">" => GoalOrder::Greater,
                                    _ => GoalOrder::Equal,
                                },
                            );
                    },
                    option { value: "=", "Равно (=)" }
                    option { value: "<", "Меньше (<)" }
                    option { value: ">", "Больше (>)" }
                }
            }

            div { class: "grid grid-cols-2 gap-2",
                div {
                    label { class: "text-xs text-foreground/70", "Период с" }
                    input {
                        r#type: "date",
                        class: "w-full px-2 py-2 border rounded-md bg-background text-sm",
                        value: "{start_date().format(\"%Y-%m-%d\")}",
                        oninput: move |e| {
                            if let Ok(d) = chrono::NaiveDate::parse_from_str(&e.value(), "%Y-%m-%d") {
                                start_date.set(naive_day_start(d));
                            }
                        },
                    }
                }
                div {
                    label { class: "text-xs text-foreground/70", "по" }
                    input {
                        r#type: "date",
                        class: "w-full px-2 py-2 border rounded-md bg-background text-sm",
                        value: "{end_date().format(\"%Y-%m-%d\")}",
                        oninput: move |e| {
                            if let Ok(d) = chrono::NaiveDate::parse_from_str(&e.value(), "%Y-%m-%d") {
                                end_date.set(naive_day_start(d));
                            }
                        },
                    }
                }
            }

            div {
                label { class: "text-sm font-medium", "Теги (через запятую)" }
                input {
                    class: "w-full px-3 py-2 border rounded-md bg-background",
                    value: "{tags_line}",
                    oninput: move |e| tags_line.set(e.value()),
                }
            }
        }

        div { class: "flex justify-end gap-2 mt-6",
            button {
                class: "px-4 py-2 border rounded-md",
                onclick: move |_| props.on_cancel.call(()),
                "Отмена"
            }

            button {
                class: "px-4 py-2 bg-primary text-white rounded-md",
                onclick: move |_| {
                    if name().trim().is_empty() || process().trim().is_empty() {
                        return;
                    }

                    let sta = day_start_timestamp(start_date().date_naive());
                    let ena = day_end_timestamp(end_date().date_naive());

                    let tags: Vec<TagModel> = tags_line()
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|tag_name| TagModel::new(tag_name, None, "#a78bfa"))
                        .collect();

                    let desc = if description().trim().is_empty() {
                        None
                    } else {
                        Some(description().trim().to_string())
                    };

                    let goal = if let Some(mut g) = props.goal.clone() {
                        g.name = name().trim().to_string();
                        g.description = desc;
                        g.process = process().trim().to_string();
                        g.completed = completed();
                        g.ordering = ordering();
                        g.start_period_ts = sta;
                        g.end_period_ts = ena;
                        g.tags = tags;
                        g
                    } else {
                        let mut g = GoalModel::new(
                            name().trim().to_string(),
                            sta,
                            ena,
                            process().trim().to_string(),
                        );
                        g.description = desc;
                        g.completed = completed();
                        g.ordering = ordering();
                        g.tags = tags;
                        g
                    };

                    props.on_save.call(goal);
                },
                if is_edit {
                    "Сохранить"
                } else {
                    "Создать"
                }
            }
        }
    }
}
