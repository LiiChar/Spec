use std::time::Duration;

use dioxus::prelude::*;
use tokio::time::sleep;

use crate::ui::{ToastAlign, ToastType, ToasterProvider};

#[component]
pub fn Toaster() -> Element {
    let context = use_context::<ToasterProvider>();
    let mut toasts = context.toasts;
    let align = context.align;
    let t_align = match align {
        ToastAlign::LeftTop => "left-3 top-3 show-left",
        ToastAlign::LeftBottom => "left-3 bottom-3 show-left",
        ToastAlign::RightTop => "right-3 top-3 show-right",
        ToastAlign::RightBottom => "right-3 bottom-3 show-right",
    };

    rsx! {
        div {
            class: "fixed flex flex-col gap-1 justify-center z-50 {t_align}",
            {toasts.read().clone().into_iter().map(|toast| {
                let mut exit_animation = use_signal(|| String::new());
                let t = match toast.t {
                    ToastType::Info => " border-border/40",
                    ToastType::Success => " border-green-400/30",
                    ToastType::Error => " border-red-400/30",
                };

                    let te_align_show = match align {
                        ToastAlign::LeftTop => "show-right",
                        ToastAlign::LeftBottom => "show-right",
                        ToastAlign::RightTop => "show-left",
                        ToastAlign::RightBottom => "show-left",
                    };

                    let te_align_exit = match align {
                        ToastAlign::LeftTop => "hide-right",
                        ToastAlign::LeftBottom => "hide-right",
                        ToastAlign::RightTop => "hide-left",
                        ToastAlign::RightBottom => "hide-left",
                    };

                rsx! {
                  div {
                    onmounted: move |_| {
                        let t = toast.clone();
                        spawn(async move {
                            sleep(Duration::from_millis(t.timeout as u64 - 200)).await;
                            exit_animation.set(te_align_exit.to_string());
                            sleep(Duration::from_millis(200)).await;
                            let mut temp = toasts.read().cloned();
                            let index = temp.binary_search(&t).expect("Failed find toast");
                            temp.remove(index);
                            exit_animation.set("".to_string());
                            toasts.set(temp);
                        });
                    },
                    class: "{t} bg-secondary/40 p-2 border {te_align_show} text-xs rounded-lg backdrop-blur-lg {exit_animation}",
                    div {
                        class: "flex justify-between gap-2",
                        div {
                            class: "flex flex-col gap-0.5",
                            span {
                                class: "text-md",
                                "{toast.title}"
                            }
                            span {
                                class: "text-muted-foreground/50",
                                {
                                    match toast.description.clone() {
                                        Some(d) => d,
                                        None => "".to_string()
                                    }
                                }
                            }
                        }
                        div {

                        }
                      }
                  }
                }
            })}
        }
    }
}
