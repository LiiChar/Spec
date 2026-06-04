use std::{collections::HashSet, time::Duration};

use dioxus::{ prelude::*};
use dioxus_free_icons::{icons::ld_icons::LdX, Icon};
use tokio::time::sleep;

use crate::ui::{AlertAlign, AlertType, Button, ButtonSize, ButtonVariant, use_alert, use_settings};

#[component]
pub fn Alerter() -> Element {
    let context = use_alert();
    let settings = use_settings();
    
    let alerts = context.alerts;
    let ctx = context.clone();

    let mut clear_ctx = ctx.clone();
    let clear = Callback::new(move |id| clear_ctx.remove(id));
    let mut cancel_ctx = ctx.clone();
    let cancel = Callback::new(move |id| cancel_ctx.cancel(id));
    let mut agree_ctx = ctx.clone();
    let agree = Callback::new(move |id| agree_ctx.ok(id));
    let align = context.align.clone();

    let closing = use_signal(HashSet::<u32>::new);

    let t_align = match align {
        AlertAlign::Left => "left-6 top-1/2  -translate-y-1/2",
        AlertAlign::Bottom => "left-1/2 bottom-6 -translate-x-1/2 ",
        AlertAlign::Right => "right-6 top-1/2 -translate-y-1/2",
        AlertAlign::Top => "left-1/2 top-6 -translate-x-1/2 -",
        AlertAlign::Center => "left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2",
    };

    let te_align_show = match align {
        AlertAlign::Left => "show-right",
        AlertAlign::Right => "show-left",
        AlertAlign::Top => "show-bottom",
        AlertAlign::Bottom => "show-top",
        AlertAlign::Center => "show",

    };

    let te_align_exit = match align {
        AlertAlign::Left => "show-left",
        AlertAlign::Right => "show-right",
        AlertAlign::Top => "show-top",
        AlertAlign::Bottom => "show-bottom",
        AlertAlign::Center => "show",
    };

    rsx! {
        div { class: "fixed flex flex-col gap-1 justify-center z-50 {t_align}",
            if settings.settings.read().enable_notifications {
                {
                    alerts
                        .read()
                        .iter()
                        .cloned()
                        .map(|alert| {
                            let border = match alert.t {
                                AlertType::Info => "border-border/40",
                                AlertType::Success => "border-green-400/30",
                                AlertType::Error => "border-red-700/30",
                            };
                            let is_closing = closing.read().contains(&alert.id);
                            rsx! {
                                div {
                                    key: "{alert.id}",

                                    class: format!(
                                        "{border} bg-secondary/30 p-2 border text-xs rounded-lg backdrop-blur-lg {te_align_show} {}",
                                        if is_closing { te_align_exit } else { "" },
                                    ),

                                    // onmounted: {
                                    //     let mut closing = closing;
                                    //     let id = alert.id;
                                    //     let timeout = alert.timeout;

                                    //     move |_| {
                                    //         spawn(async move {
                                    //             sleep(Duration::from_millis(
                                    //                 (timeout as u64).saturating_sub(200)
                                    //             )).await;

                                    //             closing.write().insert(id);
                                    //         });
                                    //     }
                                    // },
                                    div { class: "flex justify-between gap-3 items-start relative pr-[32px] group",

                                        div { class: "flex flex-col gap-0.5",

                                            span { class: "text-base", "{alert.title}" }

                                            if let Some(desc) = &alert.description {
                                                span { class: "text-xs text-muted-foreground/50", "{desc}" }
                                            }
                                        }

                                        button {
                                            class: "opacity-0 max-h-[24px] max-w-[24px] min-h-[24px] min-w-[24px] aspect-square group-hover:opacity-100 absolute right-0 top-1/2 -translate-y-1/2 z-1 rounded-full bg-secondary/40 p-0.5 text-xs text-foreground/50 hover:bg-secondary/50 hover:text-foreground/70 transition-colors",

                                            onclick: move |_| {
                                                let mut closing = closing;
                                                let id = alert.id;

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

                                    div { class: "flex justify-end gap-2 mt-3",

                                        Button {
                                            size: ButtonSize::Sm,
                                            onclick: move |_| {
                                                cancel(alert.id);
                                            },
                                            "Отмена"
                                        }

                                        Button {
                                            size: ButtonSize::Sm,
                                            variant: ButtonVariant::Error,
                                            onclick: move |_| {
                                                agree(alert.id);
                                            },
                                            "Согласиться"
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