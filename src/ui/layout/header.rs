use dioxus::{desktop::use_window, prelude::*};

#[component]
pub fn Header() -> Element {
    let window = use_window();

    let drag_window = window.clone();
    let close_window = window.clone();

    rsx! {
        div {
            class: "fixed top-0 left-0 w-full h-4 flex items-center justify-between px-4 z-50",
            
            // Левая часть - Перетаскивание окна
            div {
                onmousedown: move |_| {
                    drag_window.drag();
                },
                class: "flex-1 h-full flex items-center cursor-grab active:cursor-grabbing select-none",

            }
            
            button {
                class: "w-[28px] h-[28px] fixed top-1 hover:border border-border aspect-square rounded-full right-1 hover:bg-red-600/50 transition-colors hover:text-white hover:bg-red-600",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    close_window.close();
                },
                "✕"
            }
        }
    }
}