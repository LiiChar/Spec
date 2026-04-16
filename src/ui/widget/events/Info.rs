use dioxus::prelude::*;

use crate::core::{WindowModel};

#[component]
pub fn EventInfo(event: WindowModel) -> Element {
    rsx! {
        div { 
            class: "p-2 flex flex-col gap-4 h-full w-full",
            h2 {
                class: "text-foreground",
                "Current Window Info"
            }
            p { "Title: {event.title}" }
            p { "Path: {event.process_path}" }
            p { "Size: {event.rect.width}x{event.rect.height}" }

        }
    }
}