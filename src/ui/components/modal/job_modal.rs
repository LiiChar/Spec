use chrono::NaiveDate;
use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;
use dioxus_free_icons::Icon;

use crate::{
    core::{JobModel, with_database_mut},
    lib::convert_ts_to_local_date,
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

    let update = move |job: JobModel| {
        app.update_jobs(job);
    };

    rsx! {
        if let Some(job) = props.job {
            div {
                class: "fixed inset-0 bg-black/50 flex p-4 items-center justify-center z-[200]",
                onclick: move |_| {
                    props.on_close.call(());
                },
                div {
                    class: "bg-background p-6 rounded-lg shadow-lg max-w-96 overflow-y-auto relative job-modal-refw",
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
                    let job = job.clone();
                    rsx! {
                      JobForm {
                        job: Some(job.clone()),
                        day: job_anchor_day(&job),
                        end_ts: job.end_ts,
                        start_ts: job.start_ts,
                        on_save: move |job: JobModel| {
                            update(job.clone());
                            props.on_close.call(());
                        },
                        on_cancel: move |_| props.on_close.call(()),
                      }
                    }
                  }
                }
            }
        } else {
          ""
        }
    }
}
