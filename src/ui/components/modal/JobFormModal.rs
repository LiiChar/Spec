use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::LdX;

use crate::{core::JobModel, ui::JobForm};
use dioxus_free_icons::Icon;

#[derive(Props, PartialEq, Clone)]
pub struct JobFormModalProps {
    #[props(default = Signal::new(false))]
    pub visible: WriteSignal<bool>,
    pub start_ts: i64,
    pub end_ts: i64,
    pub on_save: Callback<JobModel>,
    #[props(default = Callback::new(|_| ()))]
    pub on_cancel: Callback<()>,
    #[props(default = Callback::new(|_| ()))]
    pub on_close: Callback<()>,
}

#[component]
pub fn JobFormModal(props: JobFormModalProps) -> Element {
    rsx! {
        if *props.visible.read() {
            div {
                class: "fixed inset-0 bg-black/50 flex p-4 items-center justify-center z-[200]",
                onclick: move |_| {
                    props.on_close.call(());
                    props.on_cancel.call(());
                    props.visible.boxed_mut().set(false);
                },
                div {
                    class: "bg-background p-6 rounded-lg shadow-lg max-w-96 overflow-y-auto",
                    onclick: move |evt| evt.stop_propagation(),
                    div {
                        class: "absolute top-2 -right-2",
                        Icon {
                            icon: LdX
                        }
                    }
                    JobForm {
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
