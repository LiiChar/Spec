use dioxus::prelude::*;

use crate::ui::{ToastAlign, ToastType, ToasterProvider};

#[component]
pub fn Toaster() -> Element {
    let context = use_context::<ToasterProvider>();
    let toasts = context.toasts;
    let align = context.align;
    let t_align = match align {
        ToastAlign::LeftTop => "-left-3 top-3",
        ToastAlign::LeftBottom => "-left-3 bottom-3",
        ToastAlign::RightTop => "right-3 top-3",
        ToastAlign::RightBottom => "right-3 bottom-3",
    };

    rsx! {
        div {
            class: "fixed flex flex-col gap-1 justify-center z-50 {t_align}",
            {toasts.iter().map(|toast| {
              let t = match toast.t {
                ToastType::Info => "bg-secondary/30 border-border/40",
                ToastType::Success => "bg-success/30 border-success/40",
                ToastType::Error => "bg-error/30 border-error/40",
              };
                rsx! {
                  div {
                      class: "{t} p-2 border text-xs backdrop-blur-md",
                      "{toast.title}"
                  }
                }
            })}
        }
    }
}
