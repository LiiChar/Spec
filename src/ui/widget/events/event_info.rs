use chrono::Datelike;
use dioxus::prelude::*;

use crate::core::WindowModel;

#[derive(Props, PartialEq, Clone)]
pub struct EventInfoProps {
    pub event: WindowModel,
}

#[component]
pub fn EventInfo(props: EventInfoProps) -> Element {
    let window = &props.event;

    rsx! {
        div { class: "hidden lg:flex flex-col gap-4 w-80 p-4 bg-zinc-900/30 rounded-lg border border-zinc-700/30",

            // Заголовок
            h2 { class: "text-lg font-bold text-cyan-400 truncate", "Текущее окно" }

            // Процесс
            div { class: "flex flex-col gap-2",

                label { class: "text-xs uppercase text-gray-400 font-bold tracking-wider",
                    "Процесс"
                }
                div { class: "text-sm font-semibold text-gray-200 truncate bg-zinc-800/50 rounded p-2",
                    "{window.process_name}"
                }
            }

            // Заголовок окна
            div { class: "flex flex-col gap-2",

                label { class: "text-xs uppercase text-gray-400 font-bold tracking-wider",
                    "Заголовок"
                }
                div {
                    class: "text-sm text-gray-300 truncate bg-zinc-800/50 rounded p-2 max-h-20 overflow-y-auto",
                    style: "scrollbar-width: none;",
                    "{window.title}"
                }
            }

            // Информация об окне
            div { class: "flex flex-col gap-2 pt-2 border-t border-zinc-700/50",

                // PID
                div { class: "flex flex-row justify-between items-center",
                    span { class: "text-xs text-gray-400", "PID" }
                    span { class: "text-sm font-semibold text-purple-400", "{window.pid}" }
                }

                // HWND
                div { class: "flex flex-row justify-between items-center",
                    span { class: "text-xs text-gray-400", "HWND" }
                    span { class: "text-sm font-semibold text-green-400 font-mono",
                        {
                            let h = format!("0x{:X}", window.hwnd);
                            h
                        }
                    }
                }

                // Размер окна
                div { class: "flex flex-row justify-between items-center",
                    span { class: "text-xs text-gray-400", "Размер" }
                    span { class: "text-sm font-semibold text-blue-400",
                        "{window.rect.width}×{window.rect.height}"
                    }
                }

                // Позиция
                div { class: "flex flex-row justify-between items-center",
                    span { class: "text-xs text-gray-400", "Позиция" }
                    span { class: "text-sm font-semibold text-pink-400",
                        "{window.rect.left}, {window.rect.top}"
                    }
                }
            }

            // Статус окна
            div { class: "flex flex-col gap-2 pt-2 border-t border-zinc-700/50",

                div { class: "flex flex-row items-center justify-between",

                    span { class: "text-xs text-gray-400", "Видимо" }

                    div {
                        class: format!(
                            "px-2 py-1 rounded text-xs font-bold {}",
                            if window.is_visible {
                                "bg-green-500/30 text-green-400"
                            } else {
                                "bg-red-500/30 text-red-400"
                            },
                        ),
                        if window.is_visible {
                            "✓"
                        } else {
                            "✗"
                        }
                    }
                }

                div { class: "flex flex-row items-center justify-between",

                    span { class: "text-xs text-gray-400", "В фокусе" }

                    div {
                        class: format!(
                            "px-2 py-1 rounded text-xs font-bold {}",
                            if window.is_focused {
                                "bg-green-500/30 text-green-400"
                            } else {
                                "bg-gray-500/30 text-gray-400"
                            },
                        ),
                        if window.is_focused {
                            "✓"
                        } else {
                            "✗"
                        }
                    }
                }

                div { class: "flex flex-row items-center justify-between",

                    span { class: "text-xs text-gray-400", "Свернуто" }

                    div {
                        class: format!(
                            "px-2 py-1 rounded text-xs font-bold {}",
                            if window.is_minimized {
                                "bg-yellow-500/30 text-yellow-400"
                            } else {
                                "bg-gray-500/30 text-gray-400"
                            },
                        ),
                        if window.is_minimized {
                            "✓"
                        } else {
                            "✗"
                        }
                    }
                }

                div { class: "flex flex-row items-center justify-between",

                    span { class: "text-xs text-gray-400", "Развернуто" }

                    div {
                        class: format!(
                            "px-2 py-1 rounded text-xs font-bold {}",
                            if window.is_maximized {
                                "bg-blue-500/30 text-blue-400"
                            } else {
                                "bg-gray-500/30 text-gray-400"
                            },
                        ),
                        if window.is_maximized {
                            "✓"
                        } else {
                            "✗"
                        }
                    }
                }
            }

            // Путь к процессу
            div { class: "flex flex-col gap-2 pt-2 border-t border-zinc-700/50",

                label { class: "text-xs uppercase text-gray-400 font-bold tracking-wider",
                    "Путь"
                }
                div {
                    class: "text-xs text-gray-400 truncate bg-zinc-800/50 rounded p-2 font-mono",
                    title: "{window.process_path}",
                    "{window.process_path}"
                }
            }
        }
    }
}
