use std::{ops::Add, time::Duration};

use chrono::{Local, Month, TimeZone, Timelike};
use dioxus::{desktop::{WindowCloseBehaviour, muda::Icon, use_window}, prelude::*};

use dioxus_free_icons::icons::ld_icons::{LdPlus, LdMinus};
use dioxus_free_icons::Icon;

use crate::ui::{AppContext, EventsCalendar, TimeInput};

#[component]
pub fn Header() -> Element {
    let window = use_window();

    let mut context = use_context::<AppContext>();
    let events = context.events;
    let day = context.day;
    let mut time = context.time;
    let mut start_time = context.start_time;



    let mut time_start = use_signal(|| {
        let now = Local::now().time();
        now.hour() * 3600 +
            now.minute() * 60 +
            now.second()
    });
    let mut time_end = use_signal(|| {
        let now = Local::now().time();

        now.hour() * 3600 +
            now.minute() * 60 +
            now.second()
    });

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

    let mut filter_time = use_signal(|| false);

    let drag_window = window.clone();
    let close_window = window.clone();

    let mut show_calendar = use_signal(|| false);

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
            class: "fixed top-0 left-0 w-full h-4 flex items-center justify-between px-4 z-50",
            
            // Левая часть - Перетаскивание окна
            div {
                onmousedown: move |_| {
                    drag_window.drag();
                },
                class: "flex-1 h-full flex items-center cursor-grab active:cursor-grabbing select-none",

            },
            div { 
                class: "fixed top-2 right-2 flex flex-row items-center z-10 gap-2 header-calendar",
                div {
                    onclick: move |evt| {
                        evt.stop_propagation();
                        evt.prevent_default();
                        if show_calendar() {
                            filter_time.set(false);
                            start_time.set(None);
                        }
                        show_calendar.set(!show_calendar());
                    },
                    class: "relative text-sm",
                    div {
                        class: "cursor-pointer rounded-full  bg-background/20 hover:bg-secondary/20 backdrop-blur-lg border border-border/30 px-2 py-1 text-sm select-none",
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
                            class: "absolute top-8 right-0 select-none show-left",
                             div { 
                                
                                class: "absolute backdrop-blur-lg cursor-pointer -top-[33px] left-0 flex gap-1  flex-row  items-center justify-center h-[30px] rounded-full bg-background/20   border border-border/30 text-sm p-0.5",
                                div {
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let naive_next_month = context.day.read().date_naive().checked_sub_months(chrono::Months::new(1)).expect("Failed sub month to current date");
                                        let cl_day = Local.from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                                        context.day.set(cl_day);
                                    },
                                    
                                    class: "flex items-center justify-center rounded-full hover:bg-secondary/20  h-full w-full p-0.5 aspect-square",
                                    "←"
                                },
                                div {
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let naive_next_month = context.day.read().date_naive().checked_add_months(chrono::Months::new(1)).expect("Failed add month to current date");
                                        let cl_day = Local.from_local_datetime(&naive_next_month.and_hms_opt(0, 0, 0).unwrap()).unwrap();
                                        context.day.set(cl_day);
                                    },
                                    class: "flex items-center justify-center rounded-full hover:bg-secondary/20  h-full w-full p-0.5 aspect-square",
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
                                
                                class: "absolute backdrop-blur-lg cursor-pointer -bottom-[33px] left-0 flex gap-1  flex-row  items-center justify-center h-[30px]  w-full",
                                if *filter_time.read() {

                                    div {
                                        class: "rounded-full bg-background/20   border border-border/30 text-sm p-0.5 show-left",
                                        TimeInput { value: time_start.clone() },
                                    }
                                    div { 
                                        onclick: move |_| {
                                            filter_time.set(false);
                                            start_time.set(None);
                                        },
                                        class: "rounded-full bg-background/20   border border-border/30 text-sm p-0.5 aspect-square min-w-[26px] flex items0-center justify-center",
                                        Icon { icon: LdMinus }
                                    },
                                    div { 
                                        class: "rounded-full bg-background/20   border border-border/30 text-sm p-0.5 show-right",
                                        TimeInput { value: time_end.clone() },
                                    }
                                } else {
                                    div { 
                                        onclick: move |_| filter_time.set(true),
                                        class: "rounded-full bg-background/20   border border-border/30 text-sm p-0.5 aspect-square min-w-[26px] flex items0-center justify-center",
                                    Icon { icon: LdPlus }
                                    },
                                }
                            } 
                        }
                    }
                },
                button {
                    class: "w-[30px] h-[30px] border-border aspect-square text-xs rounded-full bg-background/20 backdrop-blur-lg border border-border/30 transition-colors hover:text-white hover:bg-red-600 flex items-center justify-center",
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        close_window.set_close_behavior(WindowCloseBehaviour::WindowHides);
                        close_window.close();
                    },
                    span {
                        "✕"
                    }
                }
            }
        }
    }
}