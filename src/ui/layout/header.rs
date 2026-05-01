use std::{cmp::Ordering, ops::Add, time::Duration};

use chrono::{Local, Month, TimeZone, Timelike};
use dioxus::{
    desktop::{muda::Icon, use_window, WindowCloseBehaviour},
    prelude::*,
};

use dioxus_free_icons::icons::ld_icons::{LdMinus, LdPlus, LdTarget};
use dioxus_free_icons::Icon;

use crate::ui::JobFormModal;
use crate::{
    config::DATABASE_PATH,
    core::{with_database, Database, JobModel},
    ui::{AppProvider, EventsCalendar, TimeInput},
};

#[component]
pub fn Header() -> Element {
    let window = use_window();

    let mut context = use_context::<AppProvider>();
    let events = context.events;
    let day = context.day;
    let mut time = context.time;
    let mut start_time = context.start_time;

    let mut time_start = use_signal(|| {
        let now = Local::now().time();
        now.hour() * 3600 + now.minute() * 60 + now.second()
    });
    let mut time_end = use_signal(|| {
        let now = Local::now().time();

        now.hour() * 3600 + now.minute() * 60 + now.second()
    });

    let mut filter_time = use_signal(|| false);

    use_effect(move || {
        let day_dt = day.read();

        let start_of_day = day_dt
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();

        let end_time = *time_end.read() as i64;

        time.set(start_of_day + end_time * 1000);
    });

    use_effect(move || {
        let day_dt = day.read();

        if !filter_time() {
            start_time.set(None);
            return;
        }

        let start_of_day = day_dt
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp_millis();

        let st_time = *time_start.read() as i64;

        start_time.set(Some(start_of_day + st_time * 1000));
    });

    let drag_window = window.clone();
    let close_window = window.clone();

    let mut show_calendar = use_signal(|| false);
    let mut show_job_form = use_signal(|| false);

    let mut current_time = use_signal(|| chrono::Local::now().format("%H:%M:%S").to_string());

    use_effect(move || {
        spawn(async move {
            loop {
                current_time.set(chrono::Local::now().format("%H:%M:%S").to_string());
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    });

    rsx! {
        div {
            class: "fixed top-0 left-0 w-full h-3 flex items-center justify-between px-3 z-200",

            // Левая часть - Перетаскивание окна
            div {
                onmousedown: move |_| {
                    drag_window.drag();
                },
                class: "flex-1 h-full flex items-center cursor-grab active:cursor-grabbing select-none",

            },
            div {
                class: "fixed top-1.5 right-1 flex flex-row items-center z-10 gap-1 header-calendar",
                div {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        if show_calendar() {
                            filter_time.set(false);
                            start_time.set(None);
                        }
                        show_calendar.set(!show_calendar());
                    },
                    onkeydown: move |evt| {
                        match evt.key() {
                            Key::Enter => {
                                evt.stop_propagation();
                                evt.prevent_default();
                                show_calendar.set(!show_calendar());
                            },
                            Key::Escape => {
                                if show_calendar() {
                                    show_calendar.set(false);
                                }
                            },
                            _ => {}
                        }
                    },
                    class: "relative text-sm",
                    tabindex: 0,
                    role: "button",
                    aria_label: "Открыть календарь. Текущая дата",
                    div {
                        class: "cursor-pointer rounded-md bg-secondary/20 hover:bg-secondary/40 border border-border/40 px-2.5 py-1 text-xs font-medium backdrop-blur-md select-none transition-all duration-200 hover:scale-[1.02] focus:ring-2 focus:ring-primary focus:ring-offset-1",
                        match show_calendar() {
                            true => {
                                let formatted_date = context.day.read().format("%d.%m.%Y").to_string();
                                formatted_date
                            },
                            false => current_time.to_string(),
                        }
                    },

                    if show_calendar() {

                        div {
                            onmount: move |_| {
                                filter_time.set(false);
                                start_time.set(None);
                            },
                            onclick: move |evt| evt.stop_propagation(),
                            onkeydown: move |evt| {
                                if evt.key() == Key::Escape {
                                    show_calendar.set(false);
                                }
                            },
                            class: "absolute top-8 right-0 select-none show-left",
                            role: "dialog",
                            aria_label: "Календарь",
                             div {

                                class: "absolute backdrop-blur-md cursor-pointer -top-[33px] left-0 flex gap-1  flex-row  items-center justify-center min-h-[25.5px] max-h-[26px] rounded-md bg-secondary/20 border border-border/40 text-xs py-0.5",
                                button {
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let naive_next_month = context.day.read().date_naive().checked_sub_months(chrono::Months::new(1)).expect("Failed sub month to current date");
                                        let cl_day = Local.from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                                        context.day.set(cl_day);
                                    },
                                    onkeydown: move |evt| {
                                        match evt.key() {
                                            Key::Enter => {
                                                evt.prevent_default();
                                                let naive_next_month = context.day.read().date_naive().checked_sub_months(chrono::Months::new(1)).expect("Failed sub month to current date");
                                                let cl_day = Local.from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                                                context.day.set(cl_day);
                                            }
                                            _ => {}
                                        }
                                    },
                                    class: "flex items-center justify-center rounded hover:bg-primary/10 h-full w-full p-1 transition-colors focus:ring-1 focus:ring-primary",
                                    tabindex: 0,
                                    type: "button",
                                    aria_label: "Предыдущий месяц",
                                    "←"
                                },
                                button {
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let naive_next_month = context.day.read().date_naive().checked_add_months(chrono::Months::new(1)).expect("Failed add month to current date");
                                        let cl_day = Local.from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                                        context.day.set(cl_day);
                                    },
                                    onkeydown: move |evt| {
                                        match evt.key() {
                                            Key::Enter => {
                                                evt.prevent_default();
                                                let naive_next_month = context.day.read().date_naive().checked_add_months(chrono::Months::new(1)).expect("Failed add month to current date");
                                                let cl_day = Local.from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                                                context.day.set(cl_day);
                                            }
                                            _ => {}
                                        }
                                    },
                                    class: "flex items-center justify-center rounded hover:bg-primary/10 h-full w-full p-1 transition-colors focus:ring-1 focus:ring-primary",
                                    tabindex: 0,
                                    type: "button",
                                    aria_label: "Следующий месяц",
                                    "→"
                                }
                            }
                            EventsCalendar { events: events.read().clone(), day: day.read().date_naive(), onselect: move |date: chrono::NaiveDate| {
                                let cl_day = Local
                                    .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                                    .unwrap();
                                context.day.set(cl_day);
                            } }

                            div {

                                class: "absolute backdrop-blur-md cursor-pointer -bottom-[33px] left-0 flex gap-1  flex-row  items-center justify-center h-[30px]  w-full",
                                if *filter_time.read() {

                                    div {
                                        class: "rounded-md bg-secondary/30 backdrop-blur-md border border-border/40 text-xs p-0.5 show-left",
                                        TimeInput { value: time_start.clone() },
                                    }
                                    div {
                                        onclick: move |_| {
                                            filter_time.set(false);
                                            start_time.set(None);
                                        },
                                        class: "rounded-md bg-secondary/30 border border-border/40 text-xs p-0.5 aspect-square min-w-[26px] backdrop-blur-md flex items0-center justify-center hover:bg-destructive/20 transition-colors",
                                        Icon { icon: LdMinus }
                                    },
                                    div {
                                        class: "rounded-md bg-secondary/30 backdrop-blur-md border border-border/40 text-xs p-0.5 show-right",
                                        TimeInput { value: time_end.clone() },
                                    }
                                } else {
                                    div {
                                        onclick: move |_| filter_time.set(true),
                                        class: "rounded-md bg-secondary/30 backdrop-blur-md border border-border/40 text-xs p-0.5 aspect-square min-w-[26px] flex items0-center justify-center hover:bg-primary/10 transition-colors",
                                    Icon { icon: LdPlus }
                                    },
                                }
                            }

                            div {
                                onclick: move |_| show_job_form.set(true),
                                onkeydown: move |evt| {
                                    match evt.key() {
                                        Key::Enter => {
                                            evt.prevent_default();
                                            show_job_form.set(true);
                                        }
                                        _ => {}
                                    }
                                },
                                class: "w-[26px] h-[26px] absolute backdrop-blur-md bg-secondary/30 rounded-md flex justify-center items-center cursor-pointer top-0 -right-7.5  border  border-border/40 aspect-square hover:bg-primary/10 transition-colors focus:ring-2 focus:ring-primary focus:ring-offset-1",
                                tabindex: 0,
                                role: "button",
                                aria_label: "Добавить задачу",
                                Icon { icon: LdTarget }
                            }

                        }
                    }
                },
                button {
                    class: "w-[26px] h-[26px]  border-border aspect-square text-xs rounded-md bg-secondary/30 border border-border/40 transition-all hover:bg-destructive/60 flex items-center justify-center hover:scale-105 focus:ring-2 focus:ring-destructive focus:ring-offset-1",
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        close_window.set_close_behavior(WindowCloseBehaviour::WindowHides);
                        close_window.close();
                    },

                    tabindex: 0,
                    role: "button",
                    aria_label: "Закрыть приложение",
                    span {
                        "✕"
                    }
                }
            }
            JobFormModal {
                visible: show_job_form,
                start_ts: time_start.read().clone().into(),
                end_ts: time_end.read().clone().into(),
                on_save: move |job: JobModel| {
                    spawn(async move {
                        let result = tokio::task::spawn_blocking(move || {
                            with_database(|db| {
                                db.save_job(&job)
                            })
                        }).await;

                        match result {
                            Ok(Ok(id)) => println!("Saved job id: {}", id),
                            Ok(Err(e)) => println!("DB error: {:?}", e),
                            Err(e) => println!("Task error: {:?}", e),
                        }
                    });
                },
                on_cancel: move |_| {
                    show_job_form.set(false);
                }
            }
        }
    }
}
