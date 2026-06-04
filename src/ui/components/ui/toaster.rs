use std::{collections::HashSet, time::Duration};

use dioxus::{ prelude::*};
use dioxus_free_icons::{icons::ld_icons::LdX, Icon};
use tokio::time::sleep;

use crate::ui::{ToastAlign, ToastType, ToasterProvider, use_settings, use_toast};

#[component]
pub fn Toaster() -> Element {
    let context = use_toast();
    let settings = use_settings();
    
    let toasts = context.toasts;
    let mut ctx = context.clone();
    let clear = Callback::new(move |id| ctx.remove(id));
    let align = context.align.clone();

    let closing = use_signal(HashSet::<u32>::new);

    let t_align = match align {
        ToastAlign::LeftTop => "left-3 top-3",
        ToastAlign::LeftBottom => "left-3 bottom-3",
        ToastAlign::RightTop => "right-3 top-3",
        ToastAlign::RightBottom => "right-3 bottom-3",
    };

    let te_align_show = match align {
        ToastAlign::LeftTop | ToastAlign::LeftBottom => "show-right",
        ToastAlign::RightTop | ToastAlign::RightBottom => "show-left",
    };

    let te_align_exit = match align {
        ToastAlign::LeftTop | ToastAlign::LeftBottom => "hide-right",
        ToastAlign::RightTop | ToastAlign::RightBottom => "hide-left",
    };

    rsx! {
        div { class: "fixed flex flex-col gap-1 justify-center z-50 {t_align}",
            if settings.settings.read().enable_notifications {
                {
                    toasts
                        .read()
                        .iter()
                        .cloned()
                        .map(|toast| {
                            let border = match toast.t {
                                ToastType::Info => "border-border/40",
                                ToastType::Success => "border-green-400/30",
                                ToastType::Error => "border-red-400/30",
                            };
                            let is_closing = closing.read().contains(&toast.id);
                            rsx! {
                                div {
                                    key: "{toast.id}",

                                    class: format!(
                                        "{border} bg-secondary/50 p-2 border text-xs rounded-lg backdrop-blur-lg {te_align_show} {}",
                                        if is_closing { te_align_exit } else { "" },
                                    ),

                                    onmounted: {
                                        let mut closing = closing;
                                        let id = toast.id;
                                        let timeout = toast.timeout;

                                        move |_| {
                                            spawn(async move {
                                                sleep(Duration::from_millis((timeout as u64).saturating_sub(200))).await;
                                                closing.write().insert(id);
                                            });
                                        }
                                    },

                                    div { class: "flex justify-between gap-3 items-start relative pr-[32px] group",

                                        div { class: "flex flex-col gap-0.5",

                                            span { class: "text-sm font-medium", "{toast.title}" }

                                            if let Some(desc) = &toast.description {
                                                span { class: "text-muted-foreground/50", "{desc}" }
                                            }
                                        }

                                        button {
                                            class: "opacity-0 max-h-[24px] max-w-[24px] min-h-[24px] min-w-[24px] aspect-square group-hover:opacity-100 absolute right-0 top-1/2 -translate-y-1/2 z-1 rounded-full bg-secondary/40 p-0.5 text-xs text-foreground/50 hover:bg-secondary/50 hover:text-foreground/70 transition-colors",

                                            onclick: move |_| {
                                                let mut closing = closing;
                                                let id = toast.id;

                                                if closing.read().contains(&id) {
                                                    return;
                                                }

                                                closing.write().insert(id);

                                                spawn(async move {
                                                    sleep(Duration::from_millis(200)).await;
                                                    clear(id)
                                                });
                                            },

                                            Icon { icon: LdX }
                                        }
                                    }
                                }
                            }
                        })
                }
            } else {
                ""
            }
        
        }
    }
}