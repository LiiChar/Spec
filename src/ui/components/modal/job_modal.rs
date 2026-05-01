use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;

use crate::{
    config::DATABASE_PATH,
    core::{Database, JobModel},
    lib::{convert_ts_to_local_date, get_start_day_ts},
    ui::JobForm,
};
use dioxus_free_icons::Icon;

#[derive(Props, PartialEq, Clone, Debug)]
pub struct JobModalProps {
    #[props(default = None)]
    pub job: Option<JobModel>,
    #[props(default = Callback::new(|_| ()))]
    pub on_close: Callback<()>,
}

#[component]
pub fn JobModal(props: JobModalProps) -> Element {
    rsx! {
        if let Some(job) = props.job {
            div {
                class: "fixed inset-0 bg-black/50 flex p-4 items-center justify-center z-[200]",
                onclick: move |_| {
                    props.on_close.call(());
                },
                div {
                    class: "bg-background p-6 rounded-lg shadow-lg max-w-96 overflow-y-auto relative",
                    style: format!("outline: 2px solid {}", job.color),
                    onclick: move |evt| evt.stop_propagation(),
                    button {
                        onclick: move |_| {
                            props.on_close.call(());
                        },
                        class: "absolute top-0 right-0 hover:bg-destructive rounded-lg p-1 transition-colors",
                        Icon {
                            icon: LdX
                        }
                    }
                  {
                    let ts = get_start_day_ts();

                    let job = job.clone();

                    let start_ts = convert_ts_to_local_date((ts + job.start_ts * 1000).try_into().unwrap());
                    let end_ts = convert_ts_to_local_date((ts + job.end_ts * 1000).try_into().unwrap());

                    rsx! {
                      JobForm { job: Some(job.clone()), end_ts: job.end_ts, start_ts: job.start_ts, on_save: move |job: JobModel| {
                        spawn(async move {
                            let result = tokio::task::spawn_blocking(move || {
                                let db = Database::new(DATABASE_PATH);
                                db.update_job(&job)
                            }).await;

                            match result {
                                Ok(Ok(id)) => println!("Saved job id: {}", id),
                                Ok(Err(e)) => println!("DB error: {:?}", e),
                                Err(e) => println!("Task error: {:?}", e),
                            }
                        });
                    }, on_cancel:  |_| () }
                    }
                  }
                }
            }
        } else {
          ""
        }
    }
}
