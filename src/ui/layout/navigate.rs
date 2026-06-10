use dioxus::prelude::*;

use crate::ui::{AppProvider, Page, tooltip::Tooltip, tooltip::TooltipAlign, use_app};
use dioxus_free_icons::{
    icons::ld_icons::{LdAreaChart, LdHome, LdMenu, LdSettings, LdX},
    Icon,
};

#[component]
pub fn Navigate() -> Element {
    let context = use_app();
    let mut page = context.page;
    let mut visible = use_signal(|| false);

    rsx! {
        div { class: "fixed z-100 bottom-3 right-3 flex flex-col gap-1 p-1  items-center rounded-full glass",
            button {
                onclick: move |_| {
                    let t_visible = visible.read().clone();
                    visible.set(!t_visible);
                },
                class: "flex flex-row gap-1 items-center w-[20px] h-[20px] cursor-pointer relative z-1",
                if !visible() {
                    Icon { icon: LdMenu }
                } else {
                    Icon { icon: LdX }
                }
            }
            if visible.read().clone() {
                div { class: "absolute -translate-y-[100%] -mt-2 p-1 py-2 -top-0 left-0 flex flex-col gap-2 rounded-lg glass show-bottom",
                    Tooltip {
                        text: "Главная странице",
                        align: TooltipAlign::Left,
                        button {
                            class: format!(
                                "w-[20px] h-[20px] cursor-pointer rounded {}",
                                if page() == Page::Main {
                                    "stroke-primary shadow-lg relative z-1"
                                } else {
                                    "stroke-foreground"
                                },
                            ),
                            onclick: move |_| {
                                page.set(Page::Main);
                                visible.set(false);
                            },
                            Icon {
                                icon: LdHome,
                                class: format!(
                                    "{}",
                                    if page() == Page::Main { "stroke-primary" } else { "stroke-foreground" },
                                ),
                            }
                        }
                    }
                    Tooltip {
                        text: "Статистика",
                        align: TooltipAlign::Left,
                        button {
                            class: format!(
                                "w-[20px] h-[20px] cursor-pointer rounded {}",
                                if page() == Page::Statistics {
                                    "stroke-primary shadow-lg relative z-1"
                                } else {
                                    "stroke-foreground"
                                },
                            ),
                            onclick: move |_| {
                                page.set(Page::Statistics);
                                visible.set(false);
                            },
                            Icon {
                                icon: LdAreaChart,
                                class: format!(
                                    "{}",
                                    if page() == Page::Statistics { "stroke-primary" } else { "stroke-foreground" },
                                ),
                            }
                        }
                    }
                    Tooltip { text: "Настройки", align: TooltipAlign::Left,
                        button {
                            class: format!(
                                "w-[20px] h-[20px] cursor-pointer rounded {}",
                                if page() == Page::Settings {
                                    "stroke-primary shadow-lg relative z-1"
                                } else {
                                    "stroke-foreground"
                                },
                            ),
                            onclick: move |_| {
                                page.set(Page::Settings);
                                visible.set(false);
                            },
                            Icon {
                                icon: LdSettings,
                                class: format!(
                                    "{}",
                                    if page() == Page::Settings { "stroke-primary" } else { "stroke-foreground" },
                                ),
                            }
                        }
                    }
                }
            }
        }
    }
}
