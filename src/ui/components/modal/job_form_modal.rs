use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;

use dioxus_free_icons::Icon;

use crate::core::JobModel;
use crate::ui::components::forms::job_form::JobForm;
use crate::ui::context::use_app;

#[derive(Props, PartialEq, Clone)]
pub struct JobFormModalProps {
    #[props(default = Signal::new(false))]
    pub visible: WriteSignal<bool>,
    pub start_ts: i64,
    pub end_ts: i64,
    pub on_save: Callback<JobModel>,
    #[props(default = Callback::new(|_| ()))]
    pub on_cancel: Callback<()>,
}

#[component]
pub fn JobFormModal(props: JobFormModalProps) -> Element {
    let context = use_app();
    let day = context.day.read().date_naive();

    rsx! {
        if *props.visible.read() {
            div {
                class: "fixed inset-0 bg-black/50 flex p-4 items-center justify-center z-[200]",
                onclick: move |_| {
                    props.on_cancel.call(());
                    props.visible.boxed_mut().set(false);
                },
                div {
                    class: "bg-background p-6 rounded-lg shadow-lg max-w-96 overflow-y-auto relative",
                    onclick: move |evt| evt.stop_propagation(),
                    button {
                        onclick: move |_| {
                            props.on_cancel.call(());
                        },
                        class: "absolute top-3 right-2 hover:bg-destructive rounded-full p-1 transition-colors",
                        Icon { icon: LdX }
                    }
                    JobForm {
                        day,
                        start_ts: props.start_ts,
                        end_ts: props.end_ts,
                        on_save: props.on_save,
                        on_cancel: props.on_cancel,
                    }
                }
            }
        }
    }
}
