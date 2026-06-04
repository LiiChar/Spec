use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdPencil, LdTrash, LdX};
use dioxus_free_icons::Icon;

use crate::lib::{event_stats, format_duration_short, get_start_day_ts};
use crate::ui::{Button, use_alert, use_toast};
use crate::{
    core::{EventModel, JobModel, with_database, with_database_mut},
    lib::{convert_ts_to_date, convert_ts_to_local_date},
    ui::{JobForm, use_app},
};

fn job_anchor_day(job: &JobModel) -> NaiveDate {
    convert_ts_to_local_date((job.start_ts as i64 * 1000) as u64).date_naive()
}

#[derive(Props, PartialEq, Clone, Debug)]
pub struct JobModalProps {
    #[props(default = None)]
    pub job: Option<JobModel>,
    #[props(default = Callback::new(|_| ()))]
    pub on_close: Callback<()>,
}

#[component]
pub fn JobModal(props: JobModalProps) -> Element {

    let app = use_app();
    let mut toast = use_toast();
    let mut alert = use_alert();

    let op_job = props.job.clone();

    let mut visible_update = use_signal(|| false);

    let u_app = app.clone();
    let r_app = app.clone();
    
    let update = move |job: JobModel| {
        u_app.update_jobs(job);
    };

    let d_app = app.clone();




    let job: JobModel = match op_job.clone() {
        Some(j) => j,
        None => return rsx! { "" },
    };

    let delete = Callback::new(move |_: ()| {
        let c_app = d_app.clone();
        let mut c_toast = toast.clone();
        alert.error("Вы уверены?".to_string(), None, None, Callback::new(move |ok| 
            if ok {
                c_app.delete_job(job.id.unwrap());
                c_toast.info("Задача успешно удалена".to_string(), None, None);
                props.on_close.call(());
            }
        ));
    });

    let events  = {
        let res_evt: Vec<EventModel> = with_database(|db| {
            db.get_events_in_range(job.start_ts, job.end_ts).unwrap_or(Vec::new())
        });
        
        res_evt
    };

    let formatted_start_job = convert_ts_to_local_date(job.start_ts as u64).format("%H:%M:%S").to_string();
    let formatted_end_job = convert_ts_to_local_date(job.end_ts as u64).format("%H:%M:%S").to_string();

    let stats = use_signal(|| event_stats(events.clone()));

    rsx! {
        div {
            class: "fixed inset-0 bg-black/50 flex p-4 items-center justify-center z-[200]",
            onclick: move |_| {
                props.on_close.call(());
            },
            div {
                class: "bg-background p-6 rounded-lg shadow-lg max-w-96 h-full overflow-y-auto relative job-modal-ref",
                style: format!("border-bottom: 2px solid {}", job.color),
                onclick: move |evt| evt.stop_propagation(),
                button {
                    onclick: move |_| {
                        props.on_close.call(());
                    },
                    class: "absolute top-1.5 right-0.5 hover:bg-destructive rounded-lg p-1 transition-colors",
                    Icon { icon: LdX }
                }
                div {
                    if !visible_update() {

                        div {
                            div { class: "flex items-center gap-2 mb-1 justify-between border-b border-border/40 pb-1",
                                h2 { class: " text-xl", "{job.name}" }
                                div { class: "text-xs text-muted-foreground/60 flex gap-2 items-center",
                                    span { "{formatted_start_job}" }
                                    span { "-" }
                                    span { "{formatted_end_job}" }
                                }
                            }
                            if let Some(desc) = &job.description {
                                p { class: "text-xs text-muted-foreground/60 mt-2",
                                    "Описание"
                                }
                                p { class: "-mt-1", "{desc}" }
                            }
                            div { class: "mt-4",

                                {
                                    let s = stats();
                                    let format_active_percent = format!("{:.1}%", s.active_percent);
                                    let format_idle_percent = format!("{:.1}%", s.idle_percent);

                                    rsx! {
                                        div { class: "",

                                            div { class: "grid grid-cols-2 gap-2 text-sm",

                                                div { class: "rounded-md border border-border/40 p-2",
                                                    div { class: "text-xs opacity-70", "Общее время" }
                                                    div { class: "font-medium", "{format_duration_short(s.total_time)}" }
                                                }

                                                div { class: "rounded-md border border-border/40 p-2",
                                                    div { class: "text-xs opacity-70", "Активность" }
                                                    div { class: "font-medium",
                                                        "{format_duration_short(s.active_time)} ({format_active_percent})"
                                                    }
                                                }

                                                div { class: "rounded-md border border-border/40 p-2",
                                                    div { class: "text-xs opacity-70", "Простой" }
                                                    div { class: "font-medium", "{format_duration_short(s.idle_time)} ({format_idle_percent})" }
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
                                                    div { class: "font-medium", "{format_duration_short(s.avg_event_duration)}" }
                                                }
                                            }

                                            if let Some(app) = &s.most_used_app {
                                                {
                                                    let act_per = format!("{:.1}%", app.active_percent);
                                                    rsx! {
                                                        div { class: "rounded-md border border-border/40 p-3 my-2",

                                                            div { class: "text-xs opacity-70 mb-1", "Самое используемое приложение" }

                                                            div { class: "font-medium", "{app.name}" }

                                                            div { class: "text-sm opacity-80",
                                                                "{format_duration_short(app.total_time)} · {act_per} активного времени"
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if !s.app_list.is_empty() {
                                                div { class: "flex flex-col gap-1",
                                                    for app in s.app_list.iter().take(5) {
                                                        div { class: "rounded-md border border-border/40 p-2",

                                                            div { class: "flex justify-between gap-2",

                                                                div { class: "truncate text-sm font-medium", "{app.name}" }

                                                                div { class: "text-xs opacity-70 whitespace-nowrap",
                                                                    "{format_duration_short(app.total_time)}"
                                                                }
                                                            }

                                                            div { class: "mt-1 h-1.5 rounded-full bg-muted overflow-hidden",

                                                                div {
                                                                    class: "h-full rounded-full bg-primary",
                                                                    style: format!("width: {:.2}%", app.active_percent),
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

                        div { class: "flex gap-1 justify-end mt-2",
                            Button {
                                onclick: move |_| {
                                    delete(());
                                },
                                Icon { icon: LdTrash }
                            }
                            Button {
                                onclick: move |_| {
                                    visible_update.set(true);
                                },
                                Icon { icon: LdPencil }
                            }
                        }
                    }
                }
                if visible_update() {
                    {
                        let job = job.clone();
                        rsx! {
                            JobForm {
                                job: Some(job.clone()),
                                day: job_anchor_day(&job),
                                end_ts: job.end_ts,
                                start_ts: job.start_ts,
                                on_save: move |job: JobModel| {
                                    update(job.clone());
                                    visible_update.set(false);
                                },
                                on_cancel: move |_| visible_update.set(false),
                            }
                        }
                    }
                }
            
            }
        
        }
    }
    }
